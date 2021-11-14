use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct CommonConfig {
    /// Directory which will be read recursively for files with toml frontmatter
    pub base_dir: String,
    /// Regex to which all files which should be read must match
    pub include: Option<String>,
    /// Regex to which all files which should not be read must match. If a file
    /// matches both `include` and `exclude`, it will be excluded
    pub exclude: Option<String>,
}

impl Default for CommonConfig {
    fn default() -> Self {
        Self {
            base_dir: "./".to_string(),
            include: Some(r"\.(md|markdown)$".to_string()),
            exclude: None,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LocalPoacherConfig {
    pub events: CommonConfig,
    pub groups: CommonConfig,
    pub initiatives: CommonConfig,
}

impl Default for LocalPoacherConfig {
    fn default() -> Self {
        Self {
            events: CommonConfig::default(),
            groups: CommonConfig::default(),
            initiatives: CommonConfig::default(),
        }
    }
}
