use chrono::NaiveDateTime;

use super::schema::*;

#[derive(Queryable)]
pub struct MeetupGroup {
    pub id: String,
    pub name: String,
    pub link: String,
    pub description: String,
    pub city: String,
    pub state: String,
    pub country: String,
    pub is_private: bool,
    pub member_count: i32,
    pub photo: String,
}

#[derive(Debug, Insertable)]
#[table_name = "meetup_groups"]
pub struct NewMeetupGroup {
    pub id: String,
    pub name: String,
    pub link: String,
    pub description: String,
    pub city: String,
    pub state: String,
    pub country: String,
    pub is_private: bool,
    pub member_count: i32,
    pub photo: String,
}

#[derive(Queryable, Insertable, Debug)]
#[table_name = "meetup_events"]
pub struct MeetupEvent {
    pub id: String,
    pub created: NaiveDateTime,
    pub updated: NaiveDateTime,
    pub duration: Option<i32>,
    pub name: String,
    pub status: String,
    pub time: NaiveDateTime,
    pub local_date: String,
    pub local_time: String,
    pub utc_offset: i32,
    pub is_online_event: bool,
    pub link: String,
    pub description: Option<String>,
    pub how_to_find_us: Option<String>,
    pub visibility: String,
    pub member_pay_fee: bool,
    pub venue_visibility: String,
    pub group_id: String,
}
