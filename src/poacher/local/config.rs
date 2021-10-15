use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct CommonConfig {
    pub base_dir: String,
    pub include_files: String,
    pub exclude_files: String,
}

impl Default for CommonConfig {
    fn default() -> Self {
        Self {
            base_dir: "./".to_string(),
            include_files: "*".to_string(),
            exclude_files: "".to_string(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub events: CommonConfig,
    pub groups: CommonConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            events: CommonConfig::default(),
            groups: CommonConfig::default(),
        }
    }
}
