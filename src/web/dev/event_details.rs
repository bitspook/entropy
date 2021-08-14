use std::path::{Path, PathBuf};

use anyhow::{bail, Error, Result};
use diesel::prelude::*;
use rocket::{local::asynchronous::Client, Route};
use rocket_dyn_templates::Template;
use rocket_sync_db_pools::diesel;
use serde::Serialize;
use serde_json::json;

use super::{EntropyDb, EntropyWebResult};
use crate::MeetupEvent;

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
async fn event_details(event_slug: String, db: EntropyDb) -> EntropyWebResult<Template> {
    use crate::db::schema::meetup_events::dsl::*;

    let event: MeetupEvent = db
        .run(|conn| {
            meetup_events
                .filter(slug.eq(event_slug))
                .first::<MeetupEvent>(conn)
        })
        .await
        .map_err(Error::from)?;

    let event: Event = event.into();
    let context = json!({ "event": event });

    Ok(Template::render("event-details", context))
}

pub fn routes() -> Vec<Route> {
    routes![event_details]
}

async fn build_one(client: std::sync::Arc<Client>, url: String, dist_dir: PathBuf) -> Result<()> {
    let dist_dir = dist_dir.as_path();

    debug!("Creating directory {}", dist_dir.display());
    if let Err(err) = tokio::fs::create_dir_all(dist_dir).await {
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
        .get(url.clone())
        .dispatch()
        .await
        .into_string()
        .await
        .ok_or(Error::msg(format!("Failed to get HTML for {}", url)))?;

    let dist_filepath = dist_dir.join("index.html");
    let dist_filepath = dist_filepath.as_path();
    debug!("Writing HTML for {} to {}", url, dist_filepath.display());
    tokio::fs::write(dist_filepath, html).await.map_err(Error::from)
}

pub async fn build(client: std::sync::Arc<Client>, dist: &Path) -> Result<()> {
    let event_slugs: Vec<String> = {
        use crate::db::schema::meetup_events::dsl::*;

        debug!("Retrieving upcoming event slugs");
        let conn = crate::db::establish_connection();
        let today = chrono::Utc::now().naive_utc();

        meetup_events
            .filter(start_time.gt(today))
            .order(start_time.asc())
            .limit(50)
            .select(slug)
            .load::<String>(&conn)
            .map_err(anyhow::Error::from)?
    };
    debug!("Found {} upcoming events", event_slugs.len());

    let mut handles = vec![];
    for slug in event_slugs.into_iter() {
        let url = format!("/events/{}", slug);
        let dist_dir = dist.join("events").join(slug);
        let client = client.clone();
        handles.push(tokio::spawn(build_one(client, url, dist_dir)));
    }

    debug!("Waiting for {} build tasks to finish", handles.len());
    futures::future::join_all(handles).await;

    Ok(())
}
