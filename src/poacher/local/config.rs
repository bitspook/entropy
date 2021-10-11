use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct CommonConfig {
    base_dir: String,
    include_files: String,
    exclude_files: String,
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
    events: CommonConfig,
    groups: CommonConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            events: CommonConfig::default(),
            groups: CommonConfig::default(),
        }
    }
}
