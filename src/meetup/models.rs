use crate::db::schema::*;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Queryable, Insertable, Debug, Clone, Serialize, Deserialize)]
#[table_name = "meetup_groups"]
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

#[derive(Queryable, Insertable, Debug, Clone, Serialize, Deserialize)]
#[table_name = "meetup_events"]
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
