//! A Poacher when triggered, goes outside the system to poach some information.
//! e.g meetup poacher might visit meetup.com and poach upcoming events.
use self::meetup::MeetupResult;
use serde_json as json;

pub mod cli;
pub mod meetup;

#[derive(Debug)]
pub enum PoacherResult {
    Meetup(MeetupResult),
}

#[derive(Debug)]
pub enum PoacherMessage {
    ResultItem(PoacherResult),
    Error(PoacherError),
    Warning(PoacherWarning),
}

#[derive(Debug)]
pub enum PoacherError {
    HttpError(reqwest::Error),
    JsonParseError(json::Error, Option<String>),
    UnknownResponseError(String),
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

#[derive(Debug)]
pub enum PoacherWarning {
    FailedPresumption(String),
}
