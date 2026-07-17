use std::time::Duration;

mod defaults {
    use super::*;
    pub const API_PORT: u16 = 3001;
    pub const COMMIT_INTERVAL: Duration = Duration::from_secs(5);
}

#[derive(Debug, Clone)]
pub struct Config {
    /// database configuration
    pub db: DatabaseConfig,

    /// address for the api server.
    /// Default is `3001`
    pub api_port: Option<u16>,

    /// interval for exporting data into database.
    /// Default is `5s`
    pub commit_interval: Option<Duration>,
}

impl Config {
    pub fn api_port(&self) -> u16 {
        self.api_port.unwrap_or(defaults::API_PORT)
    }

    pub fn commit_interval(&self) -> Duration {
        self.commit_interval.unwrap_or(defaults::COMMIT_INTERVAL)
    }
}

#[derive(Debug, Clone, Default)]
pub struct DatabaseConfig {
    pub url: String,
    pub user: String,
    pub password: Option<String>,
    pub name: String,
}
