use serde::{Deserialize, Serialize};

use super::local::LocalPoacherConfig;
use crate::Coordinates;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GroupsBlacklist {
    pub slugs: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MeetupPoacherBlacklist {
    pub groups: GroupsBlacklist,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
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
    pub local: LocalPoacherConfig,
}

impl Default for PoacherConfig {
    fn default() -> Self {
        Self {
            meetup_com: vec![],
            local: LocalPoacherConfig::default(),
        }
    }
}
