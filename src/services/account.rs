use actix_web::{web, get, HttpRequest, HttpResponse, HttpMessage, Responder, Scope};

use crate::AppState;
use crate::models::User;

#[get("/check")]
async fn check_handler(_req: HttpRequest, _data: web::Data<AppState>) -> impl Responder {
    let json_response = serde_json::json!({
        "status":  "success"
    });
    HttpResponse::Ok().json(json_response)
}


#[get("/users/me")]
async fn get_me_handler(req: HttpRequest, data: web::Data<AppState>) -> impl Responder {
    let ext = req.extensions();
    let user_id = ext.get::<uuid::Uuid>().unwrap();

    let user = sqlx::query_as!(User, "SELECT * FROM users WHERE id = $1", user_id)
        .fetch_one(&data.db)
        .await
        .unwrap();

    let json_response = serde_json::json!({
        "status":  "success",
        "data": serde_json::json!({
            "user": &user
        })
    });

    HttpResponse::Ok().json(json_response)
}

pub fn init() -> Scope {
    web::scope("/account")
        .service(get_me_handler)
        .service(check_handler)
}
