use chrono::Utc;
use diesel::prelude::*;
use rocket::Route;
use rocket_dyn_templates::Template;
use rocket_sync_db_pools::diesel;
use serde::Serialize;
use serde_json::json;

use crate::{MeetupEvent, web::EntropyWebError};

use super::{EntropyDbConn, EntropyWebResult};

#[derive(Serialize)]
struct Event {
    title: String,
    description: Option<String>,
    start_date: String,
    start_time: String,
    end_time: String,
    charges: String,
    is_online: bool,
}

impl From<MeetupEvent> for Event {
    fn from(event: MeetupEvent) -> Event {
        let start_date = event.start_time.format("%A, %B %e").to_string();
        let start_time = event.start_time.format("%l:%M%P").to_string();
        let end_time = event.end_time.format("%l:%M%P").to_string();

        Event {
            title: event.title,
            description: event.description,
            start_date,
            start_time,
            end_time,
            charges: event
                .charges
                .map(|c| c.to_string())
                .or(Some("Free".to_string()))
                .unwrap(),
            is_online: event.is_online,
        }
    }
}

#[get("/")]
async fn events(db: EntropyDbConn) -> EntropyWebResult<Template> {
    use crate::db::schema::meetup_events::dsl::*;

    let events: Vec<MeetupEvent> = db
        .run(|conn| {
            let today = Utc::now().naive_utc();

            meetup_events
                .filter(start_time.gt(today))
                .order(start_time.asc())
                .limit(50)
                .load::<MeetupEvent>(conn)
        })
        .await
        .map_err(|e| EntropyWebError::DbError(e))?;

    let events: Vec<Event> = events.into_iter().map(|e| e.into()).collect();

    let context = json!({ "events": events });

    Ok(Template::render("events", context))
}

pub fn routes() -> Vec<Route> {
    routes![events]
}
