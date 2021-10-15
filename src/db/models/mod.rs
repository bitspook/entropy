use super::schema::*;
use chrono::NaiveDateTime;
use uuid::Uuid;

mod event;
mod group;

pub use event::*;
pub use group::*;

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
