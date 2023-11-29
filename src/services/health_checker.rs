use actix_web::{web, get, HttpResponse, Responder, Scope};

#[get("/healthchecker")]
async fn health_checker_handler() -> impl Responder {
    const MESSAGE: &str = "Healthcheck OK";

    HttpResponse::Ok().json(serde_json::json!({"status": "success", "message": MESSAGE}))
}

pub fn init() -> Scope {
    web::scope("/service")
        .service(health_checker_handler)
}
