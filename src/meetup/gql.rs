use reqwest::header;

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
