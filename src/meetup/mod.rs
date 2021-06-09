use reqwest::{self, header};
use serde::{Deserialize, Serialize};
use serde_json as json;
use tokio::sync::mpsc::Sender;
use url::Url;
use urlencoding;

use crate::ScraperMessage;

#[derive(Debug, Serialize, Deserialize)]
pub struct MeetupGroup {
    id: String,
    name: String,
    link: Url,
    description: String,
    city: String,
    state: String,
    country: String,
    is_private: bool,
    member_count: u32,
    photo: Url,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MeetupEvent {
    id: String,
    title: String,
    event_url: Url,
    date_time: String,
    is_saved: bool,
    timezone: String,
    venue: Option<String>,
    is_online: bool,
    going_count: u32,
    max_tickets: u32,
}

pub fn make_meetup_image_url<'a>(base_url: &Url, id: &'a str) -> Url {
    format!("https://www.meetup.com/_next/image/?url=https%3A%2F%2Fsecure-content.meetupstatic.com%2Fimages%2Fclassic-events%2F418845892%2F1920x1080.jpg&w=1920&q=100");

    let mut url = base_url.clone();
    url.set_path(id);
    url.query_pairs_mut()
        .append_pair("url", &urlencoding::encode(base_url.as_str())[..])
        .append_pair("w", "1920")
        .append_pair("q", "100");

    url
}

fn get_gql_headers() -> header::HeaderMap {
    let mut gql_headers = header::HeaderMap::new();
    gql_headers.insert(
        "Content-Type",
        header::HeaderValue::from_static("application/json"),
    );
    gql_headers.insert(
        "apollographql-client-name",
        header::HeaderValue::from_static("nextjs-web"),
    );
    // Should probably get this afresh on every hit
    gql_headers.insert(
        "X-Meetup-View-Id",
        header::HeaderValue::from_static("06d05ba6-629d-4763-87a0-6879ac4620c4"),
    );
    gql_headers.insert(
        "Origin",
        header::HeaderValue::from_static("https://www.meetup.com"),
    );

    gql_headers
}

#[derive(Debug)]
pub enum Error<'a> {
    HttpError(reqwest::Error),
    JsonParseError(json::Error, Option<&'a str>),
    UnknownResponseError(&'a str),
}

impl<'a> From<reqwest::Error> for Error<'a> {
    fn from(e: reqwest::Error) -> Self {
        Error::HttpError(e)
    }
}

impl<'a> From<json::Error> for Error<'a> {
    fn from(e: json::Error) -> Self {
        Error::JsonParseError(e, None)
    }
}

pub async fn search_groups<'a>(
    client: &reqwest::Client,
    tx: Sender<ScraperMessage<'a>>,
) -> Result<(), Error<'a>> {
    let gql_url = "https://www.meetup.com/gql";
    let gql_headers = get_gql_headers();
    // Meetup's search is trash. A lot of meetup groups get left out when searching by location because
    // Searching for following queries give better results for meetup groups of city
    // apparently all the search terms can be given in a single query, seperated by ", "
    let search_terms = vec!["chandigarh", "tricity", "mohali", "punjab"];

    // This API don't have a pagination limit, but ~2000 records make the server give internal-server-error
    // Chandigarh has ~50 groups in total, so 1000 is a safe number to ask for
    let req_body: json::Value = json::json!({
        "operationName": "rankedGroups",
        "variables": {
            "first": 1000,
            "categoryId": null,
            "lat": 30.75,
            "lon": 76.78,
            "radius": 100,
            "query": ""
        },
       "query": include_str!("./group-search.gql")
    });

    let resp = client
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
        return Err(Error::UnknownResponseError(
            "Group search response has unexpected shape",
        ));
    }

    let group_count: u32 = json::from_value(results["count"].clone())?;

    println!("Got {} groups", group_count);

    let groups = results["edges"]
        .as_array()
        .ok_or(Error::UnknownResponseError(
            "Group search response has no edges",
        ))?;

    let results = groups.iter().map(|g| {
        let node = &g["node"];
        let photo_base_url: Url = json::from_value(node["groupPhoto"]["baseUrl"].clone())
            .map_err(|e| Error::JsonParseError(e, Some("Parsing group photo base URL")))?;
        let photo_id = node["groupPhoto"]["id"]
            .as_str()
            .ok_or(Error::UnknownResponseError("Group photo ID not found"))?;
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
        let group: Result<MeetupGroup, Error> = json::from_value(group)
            .map_err(|e| Error::JsonParseError(e, Some("Parsing Group node")));

        group
    });

    for group in results {
        match group {
            // if receiver is closed, let's panic
            Ok(group) => tx.send(ScraperMessage::ResultItem(group)).await.unwrap(),
            Err(err) => tx.send(ScraperMessage::Error(err)).await.unwrap(),
        }
    }

    Ok(())
}
