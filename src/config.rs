use std::env;

#[derive(Clone)]
pub struct Config {
    pub bot_token: String,
    pub admin_username: String,
    pub database_url: String,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            bot_token: env::var("BOT_TOKEN").expect("BOT_TOKEN must be set"),
            admin_username: env::var("ADMIN_ID")
                .expect("ADMIN_ID must be a valid integer"),
            database_url: env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
        }
    }
}