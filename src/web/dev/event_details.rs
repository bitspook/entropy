use diesel::prelude::*;
use rocket::Route;
use rocket_dyn_templates::Template;
use rocket_sync_db_pools::diesel;
use serde::Serialize;
use serde_json::json;

use crate::{MeetupEvent, web::EntropyWebError};

use super::{EntropyWebResult, EntropyDbConn};

#[derive(Serialize)]
struct Event {
    title: String,
    slug: String,
    link: String,
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
            slug: event.slug,
            link: event.link,
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

#[get("/<event_slug>")]
async fn event_details(event_slug: String, db: EntropyDbConn) -> EntropyWebResult<Template> {
    use crate::db::schema::meetup_events::dsl::*;

    let event: MeetupEvent = db
        .run(|conn| {
            meetup_events
                .filter(slug.eq(event_slug))
                .first::<MeetupEvent>(conn)
        })
        .await
        .map_err(|e| EntropyWebError::DbError(e))?;

    let event: Event = event.into();
    let context = json!({ "event": event });

    Ok(Template::render("event-details", context))
}

pub fn routes() -> Vec<Route> {
    routes![event_details]
}
