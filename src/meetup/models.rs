use serde::{Deserialize, Serialize};
use url::Url;

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
    member_count: u32,
    photo: Url,
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
