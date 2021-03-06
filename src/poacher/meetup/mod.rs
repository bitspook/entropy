use chrono::DateTime;
use log::debug;
use reqwest::{self, Client};
use serde_json as json;
use tokio::sync::mpsc::Sender;
use url::Url;

pub mod consumer;
mod models;
mod util;

pub use models::*;
use util::get_gql_headers;

use crate::Coordinates;

use self::util::make_meetup_image_url;
use super::{MeetupPoacherConfig, PoacherError, PoacherMessage, PoacherResult, PoacherWarning};

pub mod cli;

#[derive(Debug)]
pub enum MeetupResult {
    Group(MeetupGroup),
    Event(MeetupEvent),
}

pub struct Meetup {
    client: Client,
    tx: Sender<PoacherMessage>,
    config: Vec<MeetupPoacherConfig>,
}

pub const SOURCE: &str = "meetup.com";

impl Meetup {
    pub fn new(
        client: Client,
        config: Vec<MeetupPoacherConfig>,
        tx: Sender<PoacherMessage>,
    ) -> Self {
        Meetup { client, tx, config }
    }

    pub async fn search_groups_(
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
            let warning = PoacherWarning::FailedPresumption(
                "Groups search has next page to fetch".to_string(),
            );

            self.tx
                .send(PoacherMessage::Warning(warning))
                .await
                .unwrap();
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
                    debug!("Found Group: {}", group.name);
                    let item = MeetupResult::Group(group);
                    let item = PoacherResult::Meetup(item);
                    self.tx
                        .send(PoacherMessage::ResultItem(item))
                        .await
                        // if receiver is closed, let's panic
                        .unwrap();
                }
                Err(err) => {
                    self.tx.send(PoacherMessage::Error(err)).await.unwrap();
                }
            }
        }

        Ok(())
    }

    pub async fn search_groups(&self) {
        for config in self.config.to_vec().into_iter() {
            let search_terms = config.search_terms;
            let coords = config.coordinates;
            let radius = config.radius;

            for term in search_terms.iter().map(|s| s.to_owned()) {
                if let Err(err) = self.search_groups_(&coords, &term, radius).await {
                    self.tx.send(PoacherMessage::Error(err)).await.unwrap();
                };
            }
        }

        self.tx.send(PoacherMessage::End).await.unwrap();
    }

    async fn _search_events(
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
            let warning = PoacherWarning::FailedPresumption(
                "Events search has next page to fetch".to_string(),
            );
            self.tx
                .send(PoacherMessage::Warning(warning))
                .await
                .unwrap();
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
                "slug": node["slug"],
                "title": node["title"],
                "description": node["description"],
                "start_time": start_time,
                "end_time": end_time,
                "is_online": node["eventType"].as_str().unwrap() == "online",
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
                    let item = PoacherResult::Meetup(MeetupResult::Group(group));
                    self.tx
                        .send(PoacherMessage::ResultItem(item))
                        .await
                        .unwrap();
                }
                Err(err) => {
                    self.tx.send(PoacherMessage::Error(err)).await.unwrap();
                }
            }
            match event {
                Ok(event) => {
                    let item = PoacherResult::Meetup(MeetupResult::Event(event));
                    self.tx
                        .send(PoacherMessage::ResultItem(item))
                        .await
                        .unwrap();
                }
                Err(err) => {
                    self.tx.send(PoacherMessage::Error(err)).await.unwrap();
                }
            }
        }

        Ok(())
    }

    pub async fn search_events(&self) {
        for config in self.config.to_vec().into_iter() {
            let coords = config.coordinates;
            let radius = config.radius;

            if let Err(err) = self._search_events(&coords, radius).await {
                self.tx.send(PoacherMessage::Error(err)).await.unwrap();
            };
        }

        self.tx.send(PoacherMessage::End).await.unwrap();
    }
}
