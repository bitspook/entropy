use crate::{
    meetup::util::{fix_meetup_datetime, make_group_events_request, make_meetup_image_url},
    Coordinates, PoachedResult, PoacherMessage, PoacherError, ScraperWarning,
};
use chrono::DateTime;
use log::debug;
use reqwest::{self, Client};
use serde_json as json;
use tokio::sync::mpsc::Sender;
use url::Url;

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
        radius: u32,
    ) -> Result<(), PoacherError> {
        let gql_url = "https://www.meetup.com/gql";
        let gql_headers = get_gql_headers();

        // This API don't have a pagination limit, but ~2000 records make the server give internal-server-error
        // Chandigarh has ~50 groups in total, so 1000 is a safe number to ask for
        let req_body: json::Value = json::json!({
            "operationName": "groupSearch",
            "variables": {
                "first": 1000,
                "categoryId": null,
                "lat": coordinates.lat,
                "lon": coordinates.lng,
                "radius": radius,
                "query": query,
                "source": "GROUPS"
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
            return Err(PoacherError::UnknownResponseError(
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
            .ok_or(PoacherError::UnknownResponseError(
                "Group search response has no edges".to_string(),
            ))?;

        let results = groups.iter().map(|g| {
            let node = &g["node"]["result"];
            let photo_base_url: Url = json::from_value(node["groupPhoto"]["baseUrl"].clone())
                .map_err(|e| {
                    PoacherError::JsonParseError(
                        e,
                        Some("Parsing group photo base URL".to_string()),
                    )
                })?;
            let photo_id =
                node["groupPhoto"]["id"]
                    .as_str()
                    .ok_or(PoacherError::UnknownResponseError(
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
            let group: Result<MeetupGroup, PoacherError> = json::from_value(group).map_err(|e| {
                PoacherError::JsonParseError(e, Some("Parsing Group node".to_string()))
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

    async fn _fetch_group_events(
        &self,
        group_slug: String,
        group_id: String,
    ) -> Result<(), PoacherError> {
        debug!("Fetching events for group: {}", group_slug);
        let request = make_group_events_request(&self.client, group_slug.to_string());

        let resp = request.send().await?.text().await?;
        let resp: json::Value = json::from_str(&resp)?;
        let resp = resp["responses"].as_array().ok_or_else(|| {
            PoacherError::UnknownResponseError("Events response didn't return a list".to_owned())
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
                PoacherError::UnknownResponseError(
                    "Events response provided no events for group".to_owned(),
                )
            })?;
        let events = events["value"].as_array().ok_or_else(|| {
            PoacherError::UnknownResponseError("events.<slug>.value is not a list".to_owned())
        })?;
        debug!("Found {} events for {}", events.len(), group_slug);

        for event in events.iter() {
            let mut event = event.clone();
            fix_meetup_datetime(&mut event, vec!["created", "updated", "time"])?;
            event["group_id"] = json::Value::String(group_id.clone());

            let event: MeetupEvent = json::from_value(event).map_err(|err| {
                PoacherError::JsonParseError(err, Some("Converting JSON to Event".to_owned()))
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

    pub async fn search_events(
        &self,
        coordinates: &Coordinates,
        radius: u32,
    ) -> Result<(), PoacherError> {
        let gql_url = "https://www.meetup.com/gql";
        let gql_headers = get_gql_headers();

        // This API don't have a pagination limit, but ~2000 records make the server give internal-server-error
        // Chandigarh has ~50 groups in total, so 1000 is a safe number to ask for
        let req_body: json::Value = json::json!({
            "operationName": "categorySearch",
            "variables": {
                "first": 1000,
                "lat": 30.75,
                "lon": 76.78,
                "radius": radius,
                "categoryId": null,
                "startDateRange": "2021-07-18T00:21:01-04:00[Asia/Kolkata]",
                "sortField": "DATETIME"
            },
           "query": include_str!("./event-search.gql")
        });

        debug!("Searching events with coordinates: {:?}", coordinates);
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

        let results = &resp["data"]["rankedEvents"];
        if *results == json::Value::Null {
            return Err(PoacherError::UnknownResponseError(
                "Event search response has unexpected shape".to_string(),
            ));
        }

        let has_more_events: bool = json::from_value(results["pageInfo"]["hasNextPage"].clone())?;
        if has_more_events == true {
            let warning = ScraperWarning::FailedPresumption(
                "Events search has next page to fetch".to_string(),
            );
            &self.tx.send(PoacherMessage::Warning(warning));
        }

        let events = results["edges"]
            .as_array()
            .ok_or(PoacherError::UnknownResponseError(
                "Event search response has no edges".to_string(),
            ))?;

        for e in events.iter() {
            let node = &e["node"];
            let photo_base_url: Url =
                json::from_value(node["group"]["groupPhoto"]["baseUrl"].clone()).map_err(|e| {
                    PoacherError::JsonParseError(
                        e,
                        Some("Parsing group photo base URL".to_string()),
                    )
                })?;
            let photo_id = node["group"]["groupPhoto"]["id"].as_str().ok_or(
                PoacherError::UnknownResponseError("Group photo ID not found".to_string()),
            )?;
            let group_photo = make_meetup_image_url(&photo_base_url, photo_id);

            let start_time =
                node["dateTime"]
                    .as_str()
                    .ok_or(PoacherError::UnknownResponseError(format!(
                        "Got invalid data type in MeetupEvent.dateTime: {}",
                        node["dateTime"]
                    )))?;
            let start_time = DateTime::parse_from_str(start_time, "%FT%H:%M%z")
                .map_err(|e| {
                    PoacherError::UnknownResponseError(format!(
                        "Failed to parse MeetupEvent.dateTime ({}): {:?}",
                        start_time, e
                    ))
                })?
                .naive_utc();

            let end_time = node["endTime"]
                .as_str()
                .ok_or(PoacherError::UnknownResponseError(format!(
                    "Got invalid data type in MeetupEvent.endTime: {}",
                    node["endTime"]
                )))?;
            let end_time = DateTime::parse_from_str(end_time, "%FT%H:%M%z")
                .map_err(|e| {
                    PoacherError::UnknownResponseError(format!(
                        "Failed to parse MeetupEvent endTime: {:#?}",
                        e
                    ))
                })?
                .naive_utc();

            let event = json::json!({
                "id": node["id"],
                "group_slug": node["group"]["slug"],
                "title": node["title"],
                "description": node["description"],
                "start_time": start_time,
                "end_time": end_time,
                "is_online": node["eventType"].to_string() == "online",
                "charges": node["fees"],
                "curency": node["currency"],
                "link": node["eventUrl"]
            });
            let group = json::json!({
                "id": node["group"]["id"],
                "slug": node["group"]["slug"],
                "name": node["group"]["name"],
                "link": node["group"]["link"],
                "description": node["group"]["description"],
                "city": node["group"]["city"],
                "state": node["group"]["state"],
                "country": node["group"]["country"],
                "is_private": node["group"]["isPrivate"],
                "photo": group_photo,
            });
            let group: Result<MeetupGroup, PoacherError> = json::from_value(group).map_err(|e| {
                PoacherError::JsonParseError(e, Some("Parsing Event->Group node".to_string()))
            });
            let event: Result<MeetupEvent, PoacherError> = json::from_value(event).map_err(|e| {
                PoacherError::JsonParseError(e, Some("Parsing Event node".to_string()))
            });

            match group {
                Ok(group) => {
                    let item = PoachedResult::Meetup(MeetupResult::Group(group));
                    &self
                        .tx
                        .send(PoacherMessage::ResultItem(item))
                        .await
                        .unwrap();
                }
                Err(err) => {
                    &self.tx.send(PoacherMessage::Error(err)).await.unwrap();
                }
            }
            match event {
                Ok(event) => {
                    let item = PoachedResult::Meetup(MeetupResult::Event(event));
                    &self
                        .tx
                        .send(PoacherMessage::ResultItem(item))
                        .await
                        .unwrap();
                }
                Err(err) => {
                    &self.tx.send(PoacherMessage::Error(err)).await.unwrap();
                }
            }
        }

        Ok(())
    }
}

impl From<reqwest::Error> for PoacherError {
    fn from(e: reqwest::Error) -> Self {
        PoacherError::HttpError(e)
    }
}

impl From<json::Error> for PoacherError {
    fn from(e: json::Error) -> Self {
        PoacherError::JsonParseError(e, None)
    }
}
