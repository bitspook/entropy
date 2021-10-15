use crate::db::schema::*;
use chrono::NaiveDateTime;
use uuid::Uuid;

#[derive(Identifiable, Queryable, Debug)]
#[table_name = "events"]
pub struct Event {
    pub id: Uuid,
    pub title: String,
    pub slug: String,
    pub description: Option<String>,
    pub desc_format: String,
    pub group_id: Uuid,
    pub venue_id: Option<Uuid>,
    pub photos: Vec<String>,
    pub source: Option<String>,
    pub source_link: Option<String>,
    pub start_time: NaiveDateTime,
    pub end_time: NaiveDateTime,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Insertable, Debug)]
#[table_name = "events"]
pub struct NewEvent {
    pub title: String,
    pub slug: String,
    pub description: Option<String>,
    pub group_id: Option<Uuid>,
    // TODO: Refactor and make desc_format an enum
    pub desc_format: String,
    pub photos: Vec<String>,
    pub source: Option<String>,
    pub source_link: Option<String>,
    pub start_time: NaiveDateTime,
    pub end_time: NaiveDateTime,
}
