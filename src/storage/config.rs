use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct StorageCredentialsConfig {
   pub endpoint: String,
   pub region: Option<String>,
   pub access_key: String,
   pub secret_key: String,
}

impl Default for StorageCredentialsConfig {
    fn default() -> Self {
        Self {
            endpoint: String::from("http://localhost:9000"),
            region: None,
            access_key: "".to_string(),
            secret_key: "".to_string(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct StorageConfig {
    pub credentials: StorageCredentialsConfig,
    pub assets_bucket: String,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            credentials: StorageCredentialsConfig::default(),
            assets_bucket: "assets".to_string(),
        }
    }
}
