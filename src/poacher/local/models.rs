use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct LocalGroup {
    slug: String,
    name: String,
    description: String,
}

#[derive(Debug, Deserialize)]
pub struct LocalEvent {
    slug: String,
    title: String,
    description: String,
}
