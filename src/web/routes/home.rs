use anyhow::{Error, Result};
use chrono::Utc;
use diesel::prelude::*;
use rocket::{local::asynchronous::Client, Route};
use rocket_dyn_templates::Template;
use rocket_sync_db_pools::diesel;
use serde::Serialize;
use serde_json::json;

use crate::MeetupEvent;

use crate::web::{Db, WebResult};

#[derive(Serialize)]
struct Event {
    title: String,
    slug: String,
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
async fn home(db: Db) -> WebResult<Template> {
    use crate::db::schema::meetup_events::dsl::*;

    let (events, count) = db
        .run(|conn| {
            let today = Utc::now().naive_utc();

            let query = meetup_events
                .filter(start_time.gt(today))
                .order(start_time.asc());

            let events = query
                .limit(5)
                .load::<MeetupEvent>(conn)
                .map_err(Error::from)?;
            let count: i64 = query.count().get_result(conn).map_err(Error::from)?;

            let res: Result<(Vec<MeetupEvent>, i64)> = Ok((events, count));

            res
        })
        .await?;

    let events: Vec<Event> = events.into_iter().map(|e| e.into()).collect();

    let context = json!({ "events": events, "upcoming_events_count": count });

    Ok(Template::render("home", context))
}

pub fn routes() -> Vec<Route> {
    routes![home]
}

pub async fn build(client: &Client, dist: &std::path::Path) -> anyhow::Result<()> {
    let path = dist.join("index.html");

    let html = client.get("/").dispatch().await;
    let html = html
        .into_string()
        .await
        .expect("Failed to render home template");

    debug!("Writing home page to dist");
    std::fs::write(path, html)?;

    Ok(())
}
