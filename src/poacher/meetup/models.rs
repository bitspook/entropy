use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::db::models::{NewEvent, NewGroup};

#[derive(Debug, Serialize, Deserialize)]
pub struct MeetupGroup {
    pub id: String,
    pub slug: String,
    pub name: String,
    pub link: String,
    pub description: String,
    pub city: String,
    pub state: String,
    pub country: String,
    pub is_private: bool,
    pub photo: Option<String>,
}

impl From<MeetupGroup> for NewGroup {
    fn from(group: MeetupGroup) -> Self {
        Self {
            name: group.name,
            slug: group.slug,
            description: Some(group.description),
            desc_format: "text".to_string(),
            source: Some(super::SOURCE.to_string()),
            source_link: Some(group.link),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MeetupEvent {
    pub id: String,
    pub slug: String,
    pub group_slug: String,
    pub title: String,
    pub description: Option<String>,
    pub start_time: NaiveDateTime,
    pub end_time: NaiveDateTime,
    pub is_online: bool,
    pub charges: Option<f64>,
    pub currency: Option<String>,
    pub link: String,
    pub venue: Option<String>,
}

impl From<MeetupEvent> for NewEvent {
    fn from(event: MeetupEvent) -> Self {
        NewEvent {
            title: event.title,
            slug: event.slug,
            group_id: None,
            description: event.description,
            desc_format: "text".to_string(),
            photos: vec![],
            source: Some(super::SOURCE.to_string()),
            source_link: Some(event.link),
            start_time: event.start_time,
            end_time: event.end_time,
        }
    }
}
