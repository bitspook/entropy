use anyhow::Context;
use figment::{
    providers::{Env, Format, Serialized, Toml},
    Figment,
};
use serde::{Deserialize, Serialize};

use crate::{poacher::PoacherConfig, storage::StorageConfig, web::WebConfig};

#[derive(Debug, Deserialize, Serialize)]
pub struct EntropyConfig {
    pub database_url: String,
    pub poacher: PoacherConfig,
    pub rfc_dir: String,
    pub web: WebConfig,
    pub storage: StorageConfig,
}

impl Default for EntropyConfig {
    fn default() -> Self {
        Self {
            database_url: "postgresql:///entropy?host=./postgres/run".to_string(),
            web: WebConfig::default(),
            poacher: PoacherConfig::default(),
            rfc_dir: "./docs/rfcs".to_string(),
            storage: StorageConfig::default(),
        }
    }
}

impl EntropyConfig {
    pub fn load() -> anyhow::Result<EntropyConfig> {
        let config: EntropyConfig = Figment::new()
            .merge(Serialized::defaults(EntropyConfig::default()))
            .merge(Toml::file("Entropy.toml"))
            .merge(Env::prefixed("ENTROPY_").global())
            .extract()
            .with_context(|| "Invalid Entropy Configuration.")?;

        Ok(config)
    }
}
