use crate::db::schema::*;
use chrono::NaiveDateTime;
use uuid::Uuid;

#[derive(Queryable, Identifiable, Debug)]
#[table_name = "venues"]
pub struct Venue {
    id: Uuid,
    address: String,
    directions: Option<String>,
    created_at: NaiveDateTime,
    updated_at: NaiveDateTime,
}

#[derive(Insertable, Debug)]
#[table_name = "venues"]
pub struct NewVenue {
    address: String,
    directions: Option<String>,
}
