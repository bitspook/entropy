use anyhow::Context;
use figment::{
    providers::{Env, Format, Serialized, Toml},
    Figment,
};
use serde::{Deserialize, Serialize};

use crate::Coordinates;

#[derive(Debug, Deserialize, Serialize)]
pub struct StaticSiteConfig {
    pub dist_path: String,
    pub base_url: String,
}

impl Default for StaticSiteConfig {
    fn default() -> StaticSiteConfig {
        StaticSiteConfig {
            dist_path: "dist".to_string(),
            base_url: "/".to_string(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ServerConfig {
    pub port: i32,
    pub host: String,
    pub template_dir: String,
    pub static_dir: String,
    pub scss_dir: String,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            port: 8000,
            host: "127.0.0.1".to_string(),
            template_dir: "src/web/templates".to_string(),
            static_dir: "src/web/static".to_string(),
            scss_dir: "src/web/scss".to_string(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GroupsBlacklist {
    pub slugs: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MeetupPoacherBlacklist {
    pub groups: GroupsBlacklist,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MeetupPoacherConfig {
    /// Meetup's search is trash. A lot of meetup groups get left out when searching by location because
    /// Searching for following queries give better results for meetup groups of city
    /// apparently all the search terms can be given in a single query, seperated by ", "
    pub search_terms: Vec<String>,
    pub coordinates: Coordinates,
    pub radius: u32,
    pub blacklist: MeetupPoacherBlacklist,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PoacherConfig {
    pub meetup_com: Vec<MeetupPoacherConfig>,
}

impl Default for PoacherConfig {
    fn default() -> Self {
        Self { meetup_com: vec![] }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct EntropyConfig {
    pub database_url: String,
    pub static_site: StaticSiteConfig,
    pub server: ServerConfig,
    pub poacher: PoacherConfig,
    pub rfc_dir: String,
}

impl Default for EntropyConfig {
    fn default() -> Self {
        Self {
            database_url: "postgresql:///entropy?host=./postgres/run".to_string(),
            static_site: StaticSiteConfig::default(),
            server: ServerConfig::default(),
            poacher: PoacherConfig::default(),
            rfc_dir: "./docs/rfcs".to_string(),
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
