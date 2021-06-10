use crate::{Coordinates, ScraperError, ScraperMessage, ScraperResult, ScraperWarning};
use reqwest::{self, Client};
use serde_json as json;
use tokio::sync::mpsc::Sender;
use url::Url;
use urlencoding;

mod gql;
mod models;

use gql::get_gql_headers;
pub use models::*;

#[derive(Debug)]
pub enum MeetupResult {
    Group(MeetupGroup),
    Event(MeetupEvent),
}

pub struct Meetup {
    client: Client,
    tx: Sender<ScraperMessage>,
}

impl Meetup {
    pub fn new(client: Client, tx: Sender<ScraperMessage>) -> Self {
        Meetup { client, tx }
    }

    pub async fn search_groups(
        &self,
        coordinates: &Coordinates,
        query: &str,
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
                "radius": 100,
                "query": query
            },
           "query": include_str!("./group-search.gql")
        });

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
            &self.tx.send(ScraperMessage::Warning(warning));
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
                    let item = ScraperResult::Meetup(item);
                    &self
                        .tx
                        .send(ScraperMessage::ResultItem(item))
                        .await
                        // if receiver is closed, let's panic
                        .unwrap();
                }
                Err(err) => {
                    &self.tx.send(ScraperMessage::Error(err)).await.unwrap();
                }
            }
        }

        Ok(())
    }
}

pub fn make_meetup_image_url(base_url: &Url, id: &str) -> Url {
    let mut url = base_url.clone();
    url.set_path(id);
    url.query_pairs_mut()
        .append_pair("url", &urlencoding::encode(base_url.as_str())[..])
        .append_pair("w", "1920")
        .append_pair("q", "100");

    url
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
