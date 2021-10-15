use anyhow::{Error, Result};
use chrono::Utc;
use diesel::{debug_query, prelude::*};
use rocket::{local::asynchronous::Client, Route};
use rocket_dyn_templates::Template;
use rocket_sync_db_pools::diesel;
use serde::Serialize;
use serde_json::json;

use crate::db::models::Event;
use crate::EntropyConfig;

use crate::web::{Db, WebResult};

#[derive(Serialize)]
struct CtxEvent {
    title: String,
    slug: String,
    description: Option<String>,
    start_date: String,
    start_time: String,
    end_time: String,
}

impl From<Event> for CtxEvent {
    fn from(event: Event) -> CtxEvent {
        let start_date = event.start_time.format("%A, %B %e").to_string();
        let start_time = event.start_time.format("%l:%M%P").to_string();
        let end_time = event.end_time.format("%l:%M%P").to_string();

        CtxEvent {
            title: event.title,
            description: event.description,
            start_date,
            slug: event.slug,
            start_time,
            end_time,
        }
    }
}

#[get("/")]
async fn home(db: Db) -> WebResult<Template> {
    use crate::db::schema::events::dsl::*;

    let config = EntropyConfig::load()?;
    let base_url = config.static_site.base_url;

    let (events_data, count) = db
        .run(|conn| {
            let today = Utc::now().naive_utc();

            let query = events.filter(start_time.gt(today));

            debug!(
                "QUERY: [query={}]",
                debug_query::<diesel::pg::Pg, _>(&query.count())
            );

            let events_data: Vec<Event> = query
                .order(start_time.asc())
                .limit(5)
                .load(conn)
                .map_err(Error::from)?;
            let count: i64 = query.count().get_result(conn).map_err(Error::from)?;

            let res: Result<(Vec<Event>, i64)> = Ok((events_data, count));

            res
        })
        .await?;

    let events_data: Vec<CtxEvent> = events_data.into_iter().map(|e| e.into()).collect();

    let context =
        json!({ "events": events_data, "upcoming_events_count": count, "base_url": base_url });

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
