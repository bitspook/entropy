use chrono::{DateTime, Utc};
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

    pub fn slug(&self) -> String {
        self.link.path().replace("/", "")
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MeetupEvent {
    created: DateTime<Utc>,
    updated: DateTime<Utc>,
    duration: Option<i32>,
    id: String,
    name: String,
    status: String,
    time: DateTime<Utc>,
    local_date: String,
    local_time: String,
    utc_offset: i32,
    is_online_event: bool,
    link: Url,
    description: Option<String>,
    how_to_find_us: Option<String>,
    visibility: String,
    member_pay_fee: bool,
    venue_visibility: String,
}

impl MeetupEvent {
    pub fn to_db_insertable(self) -> db::models::MeetupEvent {
        db::models::MeetupEvent {
            created: self.created.naive_utc(),
            updated: self.updated.naive_utc(),
            duration: self.duration.into(),
            id: self.id,
            name: self.name,
            status: self.status,
            time: self.time.naive_utc(),
            local_date: self.local_date,
            local_time: self.local_time,
            utc_offset: self.utc_offset,
            is_online_event: self.is_online_event,
            link: self.link.to_string(),
            description: self.description,
            how_to_find_us: self.how_to_find_us,
            visibility: self.visibility,
            member_pay_fee: self.member_pay_fee,
            venue_visibility: self.venue_visibility,
        }
    }
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
