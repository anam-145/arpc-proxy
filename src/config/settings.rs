use config::{Config, ConfigError, File};
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Deserialize, Clone)]
pub struct NetworkConfig {
    pub name: String,
    pub rpc_url: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ChainConfig {
    pub name: String,
    pub protocol: String,
    pub mainnet: NetworkConfig,
    #[serde(default)]
    pub testnets: HashMap<String, NetworkConfig>,
}

impl ChainConfig {
    pub fn is_jsonrpc(&self) -> bool {
        self.protocol == "jsonrpc"
    }

    pub fn get_network(&self, network: Option<&str>) -> Option<&NetworkConfig> {
        match network {
            None => Some(&self.mainnet),
            Some(net) => self.testnets.get(net),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub server: ServerConfig,
    pub chains: HashMap<String, ChainConfig>,
}

impl Settings {
    pub fn load() -> Result<Self, ConfigError> {
        let run_mode = std::env::var("RUN_MODE").unwrap_or_else(|_| "development".into());

        let config = Config::builder()
            .add_source(File::with_name("config/default"))
            .add_source(File::with_name(&format!("config/{}", run_mode)).required(false))
            .build()?;

        config.try_deserialize()
    }

    pub fn get_chain(&self, chain_id: &str) -> Option<&ChainConfig> {
        self.chains.get(chain_id)
    }

    pub fn supported_chains(&self) -> Vec<String> {
        self.chains.keys().cloned().collect()
    }

    pub fn server_addr(&self) -> String {
        format!("{}:{}", self.server.host, self.server.port)
    }
}
