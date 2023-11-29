use actix_cors::Cors;
use actix_web::middleware::Logger;
use actix_web::{http::header, App, HttpServer, web};
use sqlx::{Postgres, Pool};
use dotenv::dotenv;

mod models;
mod api_schemas;
mod modules;
mod services;
mod shared;
mod middlewares;

use modules::{config, database, mailer};
use middlewares::jwt::AuthRequired;
use services::{health_checker, authentication, account};

pub struct AppState {
    db: Pool<Postgres>,
    mailer: mailer::Mailer,
    config: config::Config,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "actix_web=info");
    }
    dotenv().ok();

    env_logger::init();

    let config = config::Config::init();
    let mailer = mailer::Mailer::new(&config);
    let pool = database::init(&config).await;

    let host = config.backend_host.clone();
    let port = config.backend_port.clone();
    let front_url = config.front_url.clone();

    println!("🚀 Server started successfully ({}:{})", &host, &port);

    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin(&front_url)
            .allowed_methods(vec!["GET", "POST"])
            .allowed_headers(vec![
                header::CONTENT_TYPE,
                header::AUTHORIZATION,
                header::ACCEPT,
            ])
            .supports_credentials();
        App::new()
            .app_data(web::Data::new(AppState { config: config.clone(), mailer: mailer.clone(), db: pool.clone() }))
            .wrap(cors)
            .wrap(Logger::default())
            .service(health_checker::init())
            .service(authentication::init())
            .service(web::scope("/api")
                .wrap(AuthRequired)
                .service(account::init()))
    })
    .bind((host, port))?
    .run()
    .await
}