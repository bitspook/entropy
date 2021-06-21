use chrono::{DateTime, Utc};
use reqwest::{Client, RequestBuilder, Url, header};

pub fn get_gql_headers() -> header::HeaderMap {
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

pub fn make_group_events_request(client: &Client, slug: String) -> RequestBuilder {
    // let now: DateTime<Utc> = Utc::now();
    // let today = now.format("%Y-%m-%d");
    let url_str = format!("https://www.meetup.com/mu_api/urlname/events/past?queries=(\
                           endpoint:{slug}/events,\
                           list:(dynamicRef:list_events_{slug}_past_cancelled,merge:()),\
                           meta:(method:get),\
                           params:(desc:true,fields:'comment_count,event_hosts,featured_photo,plain_text_no_images_description,series,selfvenue,venue_visibility,pro_network_event',\
                           has_ended:true,\
                           page:'1000',\
                           scroll:'recent_past',\
                           status:'upcoming,past,cancelled'\
                           ),ref:events_{slug}_past_cancelled)", slug=slug);
    let url = Url::parse(&url_str).unwrap();

    client.get(url).header("accept", "application/json")
}
