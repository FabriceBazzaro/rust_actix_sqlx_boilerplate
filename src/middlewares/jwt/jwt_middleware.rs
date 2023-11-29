use std::{
    rc::Rc,
    fmt,
    future::Future,
    pin::Pin, sync::{Mutex, Arc}, collections::HashMap };
use chrono::prelude::*;
use actix_web::{
    web,
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse},
    error::{ ErrorUnauthorized, ErrorInternalServerError },
    Error,
    HttpMessage
};
use jsonwebtoken::{decode, DecodingKey, Validation, errors::ErrorKind};
use serde::Serialize;
use uuid::Uuid;
use lazy_static::lazy_static;

use crate::models::Token;
use crate::middlewares::jwt::JwtToken;
use crate::AppState;

type LocalBoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + 'a>>;

lazy_static! {
    static ref COMPUTATIONS: Arc<Mutex<HashMap<String, Arc<Mutex<(String, String)>>>>> = Arc::new(Mutex::new(HashMap::new()));
}

fn declare_lock(key: &String) -> Arc<Mutex::<(String, String)>>{
    let ref mut computation_map = (*COMPUTATIONS).lock().unwrap();

    match (*computation_map).get(key) {
        Some(pair_mutex) => {
            Arc::clone(pair_mutex)
        },
        None => {
            println!("New key: {}", key);
            let new_pair_mutex = Arc::new(Mutex::new(("".to_string(),"".to_string())));
            (*computation_map).insert(key.clone(), Arc::clone(&new_pair_mutex));
            new_pair_mutex
        }
    }
}

fn remove_lock_if_not_used(key: &String) {
    let ref mut computation_map = (*COMPUTATIONS).lock().unwrap();
    let mut need_delete = false;

    if let Some(pair_mutex) = (*computation_map).get_mut(key) {
        if let Ok(_) = pair_mutex.try_lock() {
            println!("OK pair check");
            need_delete = true;
        }
        else {
            println!("Fail pair check");
        }
    }
    if need_delete  { 
        println!("Clean key: {}", key);
        (*computation_map).remove(key);
    }
}


#[derive(Debug, Serialize)]
struct ErrorResponse {
    status: String,
    message: String,
}

impl fmt::Display for ErrorResponse {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", serde_json::to_string(&self).unwrap())
    }
}

pub struct JwtMiddleware<S> {
    pub service: Rc<S>,
}

fn generate_error() -> Error {
    let json_error = ErrorResponse {
        status: "fail".to_owned(),
        message: "You are not logged in, please provide a token".to_owned(),
    };
    ErrorUnauthorized(json_error)
}

fn generate_db_error() -> Error {
    let json_error = ErrorResponse {
        status: "error".to_owned(),
        message: "Internal server error, database access".to_owned(),
    };
    ErrorInternalServerError(json_error)
}


impl<S, B> Service<ServiceRequest> for JwtMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let svc = self.service.clone();

        Box::pin(async move {
            let data = req.app_data::<web::Data<AppState>>().unwrap().clone();
            let mut claims: JwtToken = JwtToken { iat: 0, exp: 0, user_id: Uuid::nil(), id: Uuid::nil() };
            let need_check_refresh: bool;
            let mut need_refresh: bool = false;

            let access_cookie = req.cookie("access_cookie").map(|c| c.value().to_string());
            if access_cookie.is_none() {
                return Err(generate_error());
            }
            let cookie_content = access_cookie.unwrap();
            let lock = declare_lock(&cookie_content);
            let mut result_mutex = lock.lock().unwrap();

            let access_token = decode::<JwtToken>(&cookie_content, &DecodingKey::from_secret(data.config.jwt_secret.as_ref()), &Validation::default());
            match access_token {
                Ok(c) => {
                    need_check_refresh = !Token::is_valid(c.claims.user_id, c.claims.id, &data.db).await.or_else(|_| Err(generate_db_error()))?;
                    claims = c.claims;
                },
                Err(err) => {
                    println!("data error {:?}", err);
                    match *err.kind() {
                        ErrorKind::ExpiredSignature => {
                            need_check_refresh = true;
                        },
                        _ => return Err(generate_error())
                    }
                },
            };

            if need_check_refresh {
                if result_mutex.0 == "".to_string() {
                    let refresh_cookie = req.cookie("refresh_cookie").map(|c| c.value().to_string());
                    if refresh_cookie.is_none() {
                        return Err(generate_error());
                    }

                    let decoded_refresh = decode::<JwtToken>(&refresh_cookie.unwrap(), &DecodingKey::from_secret(data.config.jwt_secret.as_ref()), &Validation::default());
                    match decoded_refresh {
                        Ok(c) => {
                            if Token::is_valid(c.claims.user_id, c.claims.id, &data.db).await.or_else(|_| Err(generate_db_error()))? {
                                claims = c.claims;
                                need_refresh = true;
                            } else {
                                return Err(generate_error());
                            }
                        },
                        Err(_) => return Err(generate_error()),
                    }
                }
                else {
                    need_refresh = true;
                }
            }

            req.extensions_mut().insert::<Uuid>(claims.user_id.to_owned());
            let fut = svc.call(req);

            if need_refresh && result_mutex.0 != "".to_string() {
                println!("Use already generated tokens");
                let mut res = fut.await?;
                res.response_mut().add_cookie(&JwtToken::rebuild_cookie_from_value( "access_cookie".to_string(), result_mutex.0.clone()))?;
                res.response_mut().add_cookie(&JwtToken::rebuild_cookie_from_value( "refresh_cookie".to_string(), result_mutex.1.clone()))?;
                drop(result_mutex);
                remove_lock_if_not_used(&cookie_content);
                Ok(res)
            }
            else if need_refresh {
                println!("Generate refreshed tokens");
                let access_token = JwtToken::generate_access_token(claims.user_id.to_owned());
                let access_cookie = access_token.generate_cookie(data.config.jwt_secret.as_ref(), "access_cookie".to_string());
                let refresh_token = JwtToken::generate_refresh_token(claims.user_id.to_owned());
                let refresh_cookie = refresh_token.generate_cookie(data.config.jwt_secret.as_ref(), "refresh_cookie".to_string());

                let _ = Token::invalidate(claims.user_id.to_owned(), claims.id, &data.db).await.or_else(|_| Err(generate_db_error()))?;
                let _ = Token::remove_expired(claims.user_id.to_owned(), &data.db).await.or_else(|_| Err(generate_db_error()))?;

                println!("Generate refreshed cookies (access :{}), (refresh: {})", access_token.id, refresh_token.id);
                Token::declare_new(access_token.user_id.clone(), access_token.id, DateTime::<Utc>::from_timestamp(access_token.exp as i64, 0).unwrap(), &data.db)
                    .await.or_else(|_| Err(generate_db_error()))?;
                Token::declare_new(refresh_token.user_id.clone(), refresh_token.id, DateTime::<Utc>::from_timestamp(refresh_token.exp as i64, 0).unwrap(), &data.db)
                    .await.or_else(|_| Err(generate_db_error()))?;

                let mut res = fut.await?;
                res.response_mut().add_cookie(&access_cookie)?;
                res.response_mut().add_cookie(&refresh_cookie)?;
                result_mutex.0 = access_cookie.value().to_string();
                result_mutex.1 = refresh_cookie.value().to_string();

                drop(result_mutex);
                remove_lock_if_not_used(&cookie_content);
                Ok(res)
            } else {
                println!("Normal return");
                drop(result_mutex);
                remove_lock_if_not_used(&cookie_content);
                fut.await.map_err(|e| e.into())
            }
        })

    }
}


