use std::time::Duration;

#[derive(Debug, Clone)]
pub struct Config {
    pub database: DatabaseConfig,
    pub api_address: String,
    pub export_interval: Duration,
}

#[derive(Debug, Clone, Default)]
pub struct DatabaseConfig {
    pub url: String,
    pub user: String,
    pub password: Option<String>,
    pub name: String,
}
