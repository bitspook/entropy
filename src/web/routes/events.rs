use std::{fs, path::Path};

use anyhow::{bail, Error, Result};
use chrono::Utc;
use diesel::prelude::*;
use rocket::{local::asynchronous::Client, Route};
use rocket_dyn_templates::Template;
use serde::Serialize;
use serde_json::json;

use crate::EntropyConfig;

use crate::db::models::Event;
use crate::web::{Db, WebResult};

#[derive(Serialize)]
struct CtxEvent {
    title: String,
    description: Option<String>,
    start_date: String,
    start_time: String,
    end_time: String,
    slug: String,
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
            start_time,
            end_time,
            slug: event.slug,
        }
    }
}

#[get("/events")]
async fn events(db: Db) -> WebResult<Template> {
    use crate::db::schema::events::dsl::*;
    let config = EntropyConfig::load()?;
    let base_url = config.static_site.base_url;

    let events_data: Vec<Event> = db
        .run(|conn| {
            let today = Utc::now().naive_utc();

            events
                .filter(start_time.gt(today))
                .order(start_time.asc())
                .limit(50)
                .load::<Event>(conn)
        })
        .await
        .map_err(anyhow::Error::from)?;

    let events_data: Vec<CtxEvent> = events_data.into_iter().map(|e| e.into()).collect();

    let context = json!({ "events": events_data, "base_url": base_url });

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
