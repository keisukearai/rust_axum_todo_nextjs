use std::env;

/// 環境変数から読む設定。読み取りと検証はここに閉じ込める。
#[derive(Debug, Clone)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub database_url: String,
    pub database_max_connections: u32,
    pub cors_origin: String,
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("環境変数 {0} が設定されていません")]
    Missing(&'static str),
    #[error("環境変数 {name} の値が不正です: {value}")]
    Invalid { name: &'static str, value: String },
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        Ok(Self {
            host: env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            port: parse_var("PORT", 8080)?,
            database_url: env::var("DATABASE_URL")
                .map_err(|_| ConfigError::Missing("DATABASE_URL"))?,
            database_max_connections: parse_var("DATABASE_MAX_CONNECTIONS", 5)?,
            cors_origin: env::var("CORS_ORIGIN")
                .unwrap_or_else(|_| "http://localhost:3000".to_string()),
        })
    }

    pub fn bind_address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

fn parse_var<T: std::str::FromStr>(name: &'static str, default: T) -> Result<T, ConfigError> {
    match env::var(name) {
        Err(_) => Ok(default),
        Ok(value) => value
            .parse()
            .map_err(|_| ConfigError::Invalid { name, value }),
    }
}
