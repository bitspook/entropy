use chrono::NaiveDateTime;
use serde::Deserialize;

use crate::db::models::{NewEvent, NewGroup};

#[derive(Debug, Deserialize)]
pub struct LocalGroup {
    pub name: String,
    pub slug: String,
    pub description: String,
}

#[derive(Debug, Deserialize)]
pub struct LocalEvent {
    pub title: String,
    pub slug: String,
    pub description: String,
    pub group_slug: String,
    pub start_time: NaiveDateTime,
    pub end_time: NaiveDateTime,
}

impl Into<NewGroup> for LocalGroup {
    fn into(self) -> NewGroup {
        NewGroup {
            name: self.name,
            slug: self.slug,
            description: Some(self.description),
            desc_format: "md".to_string(),
            source: Some(super::SOURCE.to_string()),
            source_link: None,
        }
    }
}

impl Into<NewEvent> for LocalEvent {
    fn into(self) -> NewEvent {
        NewEvent {
            title: self.title,
            slug: self.slug,
            description: Some(self.description),
            group_id: None,
            desc_format: "md".to_string(),
            photos: vec![],
            source: Some(super::SOURCE.to_string()),
            source_link: None,
            start_time: self.start_time,
            end_time: self.end_time,
        }
    }
}
