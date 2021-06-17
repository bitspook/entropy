use serde::{Deserialize, Serialize};
use url::Url;

use crate::db;

#[derive(Debug, Serialize, Deserialize)]
pub struct MeetupGroup {
    id: String,
    name: String,
    link: Url,
    description: String,
    city: String,
    state: String,
    country: String,
    is_private: bool,
    member_count: i32,
    photo: Url,
}

impl MeetupGroup {
    // TODO Implement proper db serialization instead of this shit
    pub fn to_db_insertable(self) -> db::models::NewMeetupGroup {
        db::models::NewMeetupGroup {
            id: self.id,
            name: self.name,
            link: self.link.to_string(),
            description: self.description,
            city: self.city,
            state: self.state,
            country: self.country,
            is_private: self.is_private,
            member_count: self.member_count,
            photo: self.photo.to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MeetupEvent {
    id: String,
    title: String,
    event_url: Url,
    date_time: String,
    is_saved: bool,
    timezone: String,
    venue: Option<String>,
    is_online: bool,
    going_count: u32,
    max_tickets: u32,
}
