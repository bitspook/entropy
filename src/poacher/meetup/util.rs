use reqwest::{header, Url};


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

/// For some entities, meetup.com send an image ID and a base url instead of the
/// image url itself. This function tries to contruct a URL in sunch cases to
/// make consistent, downloadoable image URLs for all domain objects within our
/// code
pub fn make_meetup_image_url(base_url: &Url, id: &str) -> Url {
    let mut url = base_url.clone();
    url.set_path(id);
    url.query_pairs_mut()
        .append_pair("url", &urlencoding::encode(base_url.as_str())[..])
        .append_pair("w", "1920")
        .append_pair("q", "100");

    url
}
