#[macro_use]
extern crate diesel;


use serde_json as json;

pub mod db;
mod meetup;

pub use meetup::{Meetup, MeetupResult, MeetupGroup, MeetupEvent};

// Ideally we should make these variants enforce valid values for lat and lng,
// but since Coordinates aren't used for anything but passing to meetup API, I
// am keeping them as loose f32s
#[derive(Debug)]
pub struct Coordinates {
    lat: f32,
    lng: f32,
}

impl Coordinates {
    pub fn new(lat: f32, lng: f32) -> Self {
        Coordinates { lat, lng }
    }
}

#[derive(Debug)]
pub enum PoachedResult {
    Meetup(MeetupResult)
}

#[derive(Debug)]
pub enum PoacherMessage {
    ResultItem(PoachedResult),
    Error(ScraperError),
    Warning(ScraperWarning)
}

#[derive(Debug)]
pub enum ScraperError {
    HttpError(reqwest::Error),
    JsonParseError(json::Error, Option<String>),
    UnknownResponseError(String),
}

#[derive(Debug)]
pub enum ScraperWarning {
    FailedPresumption(String)
}
