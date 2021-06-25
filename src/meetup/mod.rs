use crate::{Coordinates, ScraperError, PoacherMessage, PoachedResult, ScraperWarning, meetup::util::{fix_meetup_datetime, make_group_events_request, make_meetup_image_url}};
use chrono::{DateTime, NaiveDateTime, Offset, TimeZone, Utc};
use log::debug;
use reqwest::{self, Client};
use serde_json as json;
use tokio::sync::mpsc::Sender;
use url::Url;
use urlencoding;

mod models;
mod util;

pub use models::*;
use util::get_gql_headers;

#[derive(Debug)]
pub enum MeetupResult {
    Group(MeetupGroup),
    Event(MeetupEvent),
}

pub struct Meetup {
    client: Client,
    tx: Sender<PoacherMessage>,
}

impl Meetup {
    pub fn new(client: Client, tx: Sender<PoacherMessage>) -> Self {
        Meetup { client, tx }
    }

    pub async fn search_groups(
        &self,
        coordinates: &Coordinates,
        query: &str,
        radius: u32
    ) -> Result<(), ScraperError> {
        let gql_url = "https://www.meetup.com/gql";
        let gql_headers = get_gql_headers();

        // This API don't have a pagination limit, but ~2000 records make the server give internal-server-error
        // Chandigarh has ~50 groups in total, so 1000 is a safe number to ask for
        let req_body: json::Value = json::json!({
            "operationName": "rankedGroups",
            "variables": {
                "first": 1000,
                "categoryId": null,
                "lat": coordinates.lat,
                "lon": coordinates.lng,
                "radius": radius,
                "query": query
            },
           "query": include_str!("./group-search.gql")
        });

        debug!(
            "Searching groups with coordinates: {:?}, query: {}",
            coordinates, query
        );
        let resp = &self
            .client
            .post(gql_url)
            .json(&req_body)
            .headers(gql_headers)
            .send()
            .await?
            .text()
            .await?;

        let resp: json::Value = json::from_str(&resp)?;

        let results = &resp["data"]["results"];
        if *results == json::Value::Null {
            return Err(ScraperError::UnknownResponseError(
                "Group search response has unexpected shape".to_string(),
            ));
        }

        let has_more_groups: bool = json::from_value(results["pageInfo"]["hasNextPage"].clone())?;
        if has_more_groups == true {
            let warning = ScraperWarning::FailedPresumption(
                "Groups search has next page to fetch".to_string(),
            );
            &self.tx.send(PoacherMessage::Warning(warning));
        }

        let groups = results["edges"]
            .as_array()
            .ok_or(ScraperError::UnknownResponseError(
                "Group search response has no edges".to_string(),
            ))?;

        let results = groups.iter().map(|g| {
            let node = &g["node"];
            let photo_base_url: Url = json::from_value(node["groupPhoto"]["baseUrl"].clone())
                .map_err(|e| {
                    ScraperError::JsonParseError(
                        e,
                        Some("Parsing group photo base URL".to_string()),
                    )
                })?;
            let photo_id =
                node["groupPhoto"]["id"]
                    .as_str()
                    .ok_or(ScraperError::UnknownResponseError(
                        "Group photo ID not found".to_string(),
                    ))?;
            let group_photo = make_meetup_image_url(&photo_base_url, photo_id);

            let group = json::json!({
                "id": node["id"],
                "name": node["name"],
                "link": node["link"],
                "description": node["description"],
                "city": node["city"],
                "state": node["state"],
                "country": node["country"],
                "is_private": node["isPrivate"],
                "photo": group_photo,
                "member_count": node["stats"]["memberCounts"]["all"]
            });
            let group: Result<MeetupGroup, ScraperError> = json::from_value(group).map_err(|e| {
                ScraperError::JsonParseError(e, Some("Parsing Group node".to_string()))
            });

            group
        });

        for group in results {
            match group {
                Ok(group) => {
                    let item = MeetupResult::Group(group);
                    let item = PoachedResult::Meetup(item);
                    &self
                        .tx
                        .send(PoacherMessage::ResultItem(item))
                        .await
                        // if receiver is closed, let's panic
                        .unwrap();
                }
                Err(err) => {
                    &self.tx.send(PoacherMessage::Error(err)).await.unwrap();
                }
            }
        }

        Ok(())
    }

    async fn _fetch_group_events(&self, group_slug: String, group_id: String) -> Result<(), ScraperError> {
        debug!("Fetching events for group: {}", group_slug);
        let request = make_group_events_request(&self.client, group_slug.to_string());

        let resp = request.send().await?.text().await?;
        let resp: json::Value = json::from_str(&resp)?;
        let resp = resp["responses"].as_array().ok_or_else(|| {
            ScraperError::UnknownResponseError("Events response didn't return a list".to_owned())
        })?;
        let events = resp
            .iter()
            .find(|r| {
                r["ref"]
                    .as_str()
                    .unwrap()
                    .to_lowercase()
                    .contains(&format!("events_{}", group_slug.to_lowercase()))
            })
            .ok_or_else(|| {
                ScraperError::UnknownResponseError(
                    "Events response provided no events for group".to_owned(),
                )
            })?;
        let events = events["value"].as_array().ok_or_else(|| {
            ScraperError::UnknownResponseError("events.<slug>.value is not a list".to_owned())
        })?;
        debug!("Found {} events for {}", events.len(), group_slug);

        for event in events.iter() {
            let mut event = event.clone();
            fix_meetup_datetime(&mut event, vec!["created", "updated", "time"])?;
            event["group_id"] = json::Value::String(group_id.clone());

            let event: MeetupEvent = json::from_value(event).map_err(|err| {
                ScraperError::JsonParseError(err, Some("Converting JSON to Event".to_owned()))
            })?;

            let msg = PoacherMessage::ResultItem(PoachedResult::Meetup(MeetupResult::Event(event)));

            self.tx.send(msg).await.unwrap();
        }

        Ok(())
    }

    pub async fn fetch_group_events(&self, group_slug: String, group_id: String) {
        if let Err(err) = self._fetch_group_events(group_slug, group_id).await {
            self.tx.send(PoacherMessage::Error(err)).await.unwrap();
        }
    }
}

impl From<reqwest::Error> for ScraperError {
    fn from(e: reqwest::Error) -> Self {
        ScraperError::HttpError(e)
    }
}

impl From<json::Error> for ScraperError {
    fn from(e: json::Error) -> Self {
        ScraperError::JsonParseError(e, None)
    }
}
