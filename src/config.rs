use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
    pub ethereum: EthereumConfig,
    pub server: ServerConfig,
    pub indexer: IndexerConfig,
    pub backend: BackendConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub url: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RedisConfig {
    pub url: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct EthereumConfig {
    pub rpc_url: String,
    pub backup_rpc_urls: Option<String>,
    pub nbgn_contract_address: String,
    pub voucher_contract_address: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Deserialize, Clone)]
pub struct IndexerConfig {
    pub start_block: u64,
    pub poll_interval_secs: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct BackendConfig {
    pub private_key: String,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let s = Config::builder()
            .add_source(File::with_name("config/default").required(false))
            .add_source(File::with_name("config/local").required(false))
            .add_source(Environment::with_prefix("").separator("_"))
            .build()?;

        s.try_deserialize()
    }
}