use std::time::Duration;

mod defaults {
    use super::*;
    pub const AUTO_MIGRATE: bool = false;
    pub const API_PORT: u16 = 3001;
    pub const COMMIT_INTERVAL: Duration = Duration::from_secs(5);
}

#[derive(Debug, Clone)]
pub struct Config {
    /// database configuration
    pub db: DatabaseConfig,

    /// Should the hub attempt to migrate the database 
    /// if it is outdated automatically
    pub auto_migrate: bool,

    /// address for the api server.
    /// Default is `3001`
    pub api_port: u16,

    /// interval for exporting data into database.
    /// Default is `5s`
    pub commit_interval: Duration,
}

impl Default for Config {
    fn default() -> Self {
        Self { 
            db: Default::default(),
            auto_migrate: defaults::AUTO_MIGRATE,
            api_port: defaults::API_PORT, 
            commit_interval: defaults::COMMIT_INTERVAL,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct DatabaseConfig {
    pub url: String,
    pub user: String,
    pub password: Option<String>,
    pub name: String,
}
