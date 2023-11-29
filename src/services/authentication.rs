use actix_web::{cookie::{time::Duration as ActixWebDuration, Cookie}, web, post, HttpResponse, Responder, Scope};
use chrono::prelude::*;
use chrono::{Duration, Utc};

use crate::{ models::{User, Code, Token},
             api_schemas::{RegisterRequestSchema, LoginRequestSchema, ConfirmCodeRequestSchema},
             middlewares::jwt::JwtToken,
             AppState};

#[post("/register")]
async fn register_handler(
    body: web::Json<RegisterRequestSchema>,
    data: web::Data<AppState>,
) -> impl Responder {
    let app_name = data.config.app_name.clone();
    let exists: bool = User::is_user_exist(body.email.to_owned(), &data.db).await;

    let insert_user_result = if !exists {
        User::create_user(body.email.to_owned(), body.language.to_owned(), &data.db).await
    } else {
        if let Some(user) = User::get_user_from_email(body.email.to_owned(), &data.db).await {
            Ok(user)
        }
        else {
            Err(sqlx::Error::RowNotFound)
        }
    };

    if let Ok(user) = insert_user_result {
        let create_code_result = Code::create_code(user.id.to_owned(), &data.db).await;

        if let Ok(code) = create_code_result {
            if user.language_id == "fr" {
                data.mailer.send_message(body.email.to_owned(), 
                    format!("Confirmez votre enregistrement sur {}", app_name), 
                    format!("Code de validation (pour 5 minutes) : {}", code.code));
            }
            else {
                data.mailer.send_message(body.email.to_owned(), 
                    format!("Confirm your registration on {}", app_name), 
                    format!("Validation code (5 minutes): {}", code.code));
            }
            return HttpResponse::Ok().json(serde_json::json!({"status": "success"}))
        }
    }
    HttpResponse::InternalServerError()
        .json(serde_json::json!({"status": "error","message": "Internal error occured during registration request"}))
}

#[post("/resend_code")]
async fn resend_code_handler(
    body: web::Json<RegisterRequestSchema>,
    data: web::Data<AppState>,
) -> impl Responder {
    let app_name = data.config.app_name.clone();
    let query_user_result = User::get_user_from_email(body.email.to_owned(), &data.db).await;

    if let Some(user) = query_user_result {
        let create_code_result = Code::create_code(user.id.to_owned(), &data.db).await;

        if let Ok(code) = create_code_result {
            if user.language_id == "fr" {
                data.mailer.send_message(body.email.to_owned(), 
                format!("Confirmez votre authentification sur {}", app_name), 
                    format!("Nouveau code de validation (pour 5 minutes) : {}", code.code));
            }
            else {
                data.mailer.send_message(body.email.to_owned(), 
                    format!("Confirm your authentication on {}", app_name), 
                    format!("New validation code (5 minutes): {}", code.code));
            }
        }
    }
    return HttpResponse::Ok().json(serde_json::json!({"status": "success"}))
}

#[post("/confirm_code")]
async fn confirm_code_handler(
    body: web::Json<ConfirmCodeRequestSchema>,
    data: web::Data<AppState>,
) -> impl Responder {
    let app_name = data.config.app_name.clone();
    let mut code_is_valid = false;
    let query_user_result = User::get_user_from_email(body.email.to_owned(), &data.db).await;

    if let Some(user) = query_user_result {
        let query_code_result = Code::get_code_from_id(user.id.clone(), &data.db).await;
        if let Some(code) = query_code_result {
            code_is_valid = (code.code == body.code) && (Utc::now() - Duration::minutes(5) <= code.emitted_at);
        }

        if code_is_valid {
            if !user.verified {
                if User::set_email_verified(user.id.to_owned(), &data.db).await.is_err() {
                    return HttpResponse::InternalServerError()
                        .json(serde_json::json!({"status": "fail", "message": "Error during account validation database request"}));    
                }
            }

            let access_token = JwtToken::generate_access_token(user.id.clone());
            let access_cookie = access_token.generate_cookie(data.config.jwt_secret.as_ref(), "access_cookie".to_string());
            let refresh_token = JwtToken::generate_refresh_token(user.id.clone());
            let refresh_cookie = refresh_token.generate_cookie(data.config.jwt_secret.as_ref(), "refresh_cookie".to_string());
        
            Token::declare_new(access_token.user_id.clone(), access_token.id, DateTime::<Utc>::from_timestamp(access_token.exp as i64, 0).unwrap(), &data.db).await.unwrap();
            Token::declare_new(refresh_token.user_id.clone(), refresh_token.id, DateTime::<Utc>::from_timestamp(refresh_token.exp as i64, 0).unwrap(),  &data.db).await.unwrap();

            return HttpResponse::Ok()
                .cookie(access_cookie)
                .cookie(refresh_cookie)
                .json(serde_json::json!({"status": "success"}))
        }
        else {
            let tries = Code::add_try(user.id.clone(), &data.db).await;
            if tries >= data.config.max_tries {
                let create_code_result = Code::create_code(user.id.to_owned(), &data.db).await;

                if let Ok(code) = create_code_result {
                    if user.language_id == "fr" {
                        data.mailer.send_message(body.email.to_owned(), 
                            format!("Déjà trois confirmations échouées. Confirmez votre authentification sur {}", app_name), 
                            format!("Code de validation (pour 5 minutes) : {}", code.code));
                    }
                    else {
                        data.mailer.send_message(body.email.to_owned(), 
                            format!("Already three confirmations failed. Confirm your authentication on {}", app_name), 
                            format!("Validation code (5 minutes): {}", code.code));
                    }
                }
                return HttpResponse::BadRequest()
                    .json(serde_json::json!({"status": "fail", "message": "Invalid email or code", "newCode": "true"}))
        
            }
        }
    }
    HttpResponse::BadRequest()
        .json(serde_json::json!({"status": "fail", "message": "Invalid email or code"}))
}

#[post("/login")]
async fn login_handler(
    body: web::Json<LoginRequestSchema>,
    data: web::Data<AppState>,
) -> impl Responder {
    let app_name = data.config.app_name.clone();
    let query_user_result = User::get_user_from_email(body.email.to_owned(), &data.db).await;

    if let Some(user) = query_user_result {
        let create_code_result = Code::create_code(user.id.to_owned(), &data.db).await;

        if let Ok(code) = create_code_result {
            if user.language_id == "fr" {
                data.mailer.send_message(body.email.to_owned(), 
                    format!("Confirmez votre authentification sur {}", app_name), 
                    format!("Code de validation (pour 5 minutes) : {}", code.code));
            }
            else {
                data.mailer.send_message(body.email.to_owned(), 
                    format!("Confirm your authentication on {}", app_name), 
                    format!("Validation code (5 minutes): {}", code.code));
            }
        }
    }
    return HttpResponse::Ok().json(serde_json::json!({"status": "success"}))
}


#[post("/logout")]
async fn logout_handler() -> impl Responder {
    let access_cookie = Cookie::build("access_cookie", "")
        .path("/")
        .max_age(ActixWebDuration::new(-1, 0))
        .http_only(true)
        .finish();
    let refresh_cookie = Cookie::build("refresh_cookie", "")
        .path("/")
        .max_age(ActixWebDuration::new(-1, 0))
        .http_only(true)
        .finish();


    HttpResponse::Ok()
        .cookie(access_cookie)
        .cookie(refresh_cookie)
        .json(serde_json::json!({"status": "success"}))
}

pub fn init() -> Scope {
    web::scope("/auth")
        .service(register_handler)
        .service(confirm_code_handler)
        .service(login_handler)
        .service(logout_handler)
        .service(resend_code_handler)
}
