use crate::db::schema::*;
use chrono::NaiveDateTime;
use uuid::Uuid;

#[derive(Queryable, Debug)]
pub struct EventSection {
    name: String,
    description: Option<String>,
    desc_format: String,
    start_time: NaiveDateTime,
    end_time: NaiveDateTime,
    event_id: String
}

#[derive(Insertable, Debug)]
#[table_name = "event_sections"]
pub struct NewVenue {
    name: String,
    description: Option<String>,
    desc_format: String,
    start_time: NaiveDateTime,
    end_time: NaiveDateTime,
    event_id: Uuid,
}
