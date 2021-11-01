use std::path::{Path, PathBuf};

use anyhow::{bail, Error, Result};
use diesel::prelude::*;
use rocket::{local::asynchronous::Client, Route};
use rocket_dyn_templates::Template;
use rocket_sync_db_pools::diesel;
use serde::Serialize;
use serde_json::json;

use crate::{EntropyConfig, web::{Db, WebResult}};

#[derive(Queryable, Serialize)]
struct EventData {
    title: String,
    slug: String,
    link: Option<String>,
    description: Option<String>,
    desc_format: String,
    start_time: chrono::NaiveDateTime,
    group_name: Option<String>,
    duration: String,
}

#[derive(Queryable, Debug, Serialize)]
struct EventSectionData {
    name: String,
    logo: Option<String>,
    title: String,
    description: Option<String>,
    desc_format: String,
    duration: String,
}

#[get("/events/<event_slug>")]
async fn event_details(event_slug: String, db: Db) -> WebResult<Template> {
    use crate::db::schema::event_sections;
    use crate::db::schema::events::dsl::*;
    use crate::db::schema::groups;

    let config = EntropyConfig::load()?.web.static_site;
    let base_url = config.base_url;

    let e_slug = event_slug.clone();
    let event_data = db
        .run(|conn| -> Result<EventData, diesel::result::Error> {
            events
                .filter(slug.eq(e_slug))
                .left_join(groups::table.on(group_id.eq(groups::id)))
                .select((
                    title,
                    slug,
                    source_link,
                    description,
                    desc_format,
                    start_time,
                    groups::name.nullable(),
                    diesel::dsl::sql(
                        r"format('%s mins', extract(epoch from (events.end_time - events.start_time))/60)",
                    ),
                ))
                .first(conn)
        })
        .await
        .map_err(Error::from)?;

    let event_sections_data = db
        .run(
            |conn| -> Result<Vec<EventSectionData>, diesel::result::Error> {
                let query = event_sections::table
                    .inner_join(events.on(id.eq(event_sections::event_id)))
                    .select((
                        event_sections::name,
                        event_sections::logo,
                        event_sections::title,
                        event_sections::description,
                        event_sections::desc_format,
                        diesel::dsl::sql(
                            r"format('%s mins', extract(epoch from (event_sections.end_time - event_sections.start_time))/60)",
                        ),
                    ))
                    .filter(slug.eq(event_slug));

                query.get_results(conn)
            },
        )
        .await
        .map_err(Error::from)?;

    let context =
        json!({ "event": event_data, "base_url": base_url, "sections": event_sections_data });

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
    tokio::fs::write(dist_filepath, html)
        .await
        .map_err(Error::from)
}

pub async fn build(client: std::sync::Arc<Client>, dist: &Path) -> Result<()> {
    let event_slugs: Vec<String> = {
        use crate::db::schema::events::dsl::*;

        debug!("Retrieving upcoming event slugs");
        let conn = crate::db::establish_connection()?;
        let today = chrono::Utc::now().naive_utc();

        events
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
