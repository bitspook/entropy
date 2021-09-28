use std::{fs, path::Path};

use anyhow::{bail, Error, Result};
use chrono::Utc;
use diesel::prelude::*;
use rocket::{local::asynchronous::Client, Route};
use rocket_dyn_templates::Template;
use rocket_sync_db_pools::diesel;
use serde::Serialize;
use serde_json::json;

use crate::poacher::meetup::MeetupEvent;
use crate::EntropyConfig;

use crate::web::{Db, WebResult};

#[derive(Serialize)]
struct Event {
    title: String,
    description: Option<String>,
    start_date: String,
    start_time: String,
    end_time: String,
    charges: String,
    is_online: bool,
    slug: String,
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
            slug: event.slug,
        }
    }
}

#[get("/events")]
async fn events(db: Db) -> WebResult<Template> {
    use crate::db::schema::meetup_events::dsl::*;
    let config = EntropyConfig::load()?;
    let base_url = config.static_site.base_url;

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
        .map_err(anyhow::Error::from)?;

    let events: Vec<Event> = events.into_iter().map(|e| e.into()).collect();

    let context = json!({ "events": events, "base_url": base_url });

    Ok(Template::render("events", context))
}

pub fn routes() -> Vec<Route> {
    routes![events]
}

pub async fn build(client: &Client, dist: &Path) -> Result<()> {
    let url = "/events";
    let dist_dir = dist.join(Path::new(url).strip_prefix("/")?);
    let dist_dir = dist_dir.as_path();
    let dist_filepath = dist_dir.join("index.html");
    let dist_filepath = dist_filepath.as_path();

    debug!("Creating directory: {}", dist_dir.display());
    if let Err(err) = fs::create_dir_all(dist_dir) {
        match err.kind() {
            std::io::ErrorKind::AlreadyExists => {
                debug!("'{}' already exists. Ignoring.", dist_dir.display());
            }
            _ => {
                bail!(
                    "Failed to create directory ({}): {:#}",
                    dist_dir.display(),
                    err
                );
            }
        }
    }

    debug!("Building HTML for '{}'", url);
    let html = client
        .get(url)
        .dispatch()
        .await
        .into_string()
        .await
        .ok_or(Error::msg(format!("Failed to get HTML for {}", url)))?;

    debug!("Writing HTML for {} to {}", url, dist_filepath.display());
    fs::write(dist_filepath, html).map_err(Error::from)
}
