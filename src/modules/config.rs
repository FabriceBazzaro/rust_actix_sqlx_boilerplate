#[derive(Debug, Clone)]
pub struct Config {
    pub app_name: String,

    pub postgres_host: String,
    pub postgres_port: u16,
    pub postgres_user: String,
    pub postgres_pwd: String,
    pub postgres_db: String,

    pub jwt_secret: String,
    pub jwt_expires_in: String,
    pub jwt_maxage: i32,

    pub max_tries: i16,

    pub backend_host: String,
    pub backend_port: u16,

    pub front_url: String,

    pub mail_host: String,
    pub mail_port: u16,
    pub mail_auth_user: String,
    pub mail_auth_pwd: String
}

fn get_field(name: &str) -> String {
    std::env::var(name).expect(format!("{} must be set in .env file", name).as_str())
}


impl Config {
    pub fn init() -> Config {
        Config {
            app_name: get_field("APP_NAME"),
            postgres_host: get_field("POSTGRES_HOST"),
            postgres_port: get_field("POSTGRES_PORT").parse::<u16>().unwrap(),
            postgres_user: get_field("POSTGRES_USER"),
            postgres_pwd: get_field("POSTGRES_PASSWORD"),
            postgres_db: get_field("POSTGRES_DB"),
            jwt_secret: get_field("JWT_SECRET"),
            jwt_expires_in: get_field("JWT_EXPIRED_IN"),
            jwt_maxage: get_field("JWT_MAXAGE").parse::<i32>().unwrap(),
            max_tries: get_field("MAX_TRIES").parse::<i16>().unwrap(),
            backend_host: get_field("BACKEND_HOST"),
            backend_port: get_field("BACKEND_PORT").parse::<u16>().unwrap(),
            front_url: get_field("FRONT_URL"),
            mail_host: get_field("MAIL_HOST"),
            mail_port: get_field("MAIL_PORT").parse::<u16>().unwrap(),
            mail_auth_user: get_field("MAIL_AUTH_USER"),
            mail_auth_pwd: get_field("MAIL_AUTH_PWD")
        }
    }
}
