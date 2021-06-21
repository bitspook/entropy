use chrono::{DateTime, TimeZone};
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
   pub created: i64,
   pub updated: i64,
   pub duration: Option<i32>,
   pub id: String,
   pub name: String,
   pub status: String,
   pub time: i64,
   pub local_date: String,
   pub local_time: String,
   pub utc_offset: i32,
   pub is_online_event: bool,
   pub link: Url,
   pub description: Option<String>,
   pub how_to_find_us: Option<String>,
   pub visibility: String,
   pub member_pay_fee: bool,
   pub venue_visibility: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MeetupVenue {
    id: String,
    name: String,
    lat: f32,
    lon: f32,
    repinned: bool,
    city: String,
    country: String,
    localized_country_name: String,
}
