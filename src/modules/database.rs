use sqlx::{postgres::{PgPoolOptions, PgConnectOptions}, Postgres, Pool};
use crate::config::Config;

pub async fn init(config: &Config) -> Pool<Postgres> {
    let pool_options = PgConnectOptions::new()
        .host(&config.postgres_host)
        .port(config.postgres_port)
        .username(&config.postgres_user)
        .password(&config.postgres_pwd)
        .database(&config.postgres_db);

    match PgPoolOptions::new()
        .max_connections(10)
        .connect_with(pool_options)
        .await
    {
        Ok(pool) => {
            println!("✅Connection to the database is successful!");
            pool
        }
        Err(err) => {
            println!("🔥 Failed to connect to the database: {:?}", err);
            std::process::exit(1);
        }
    }
}