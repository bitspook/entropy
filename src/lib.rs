#[macro_use]
extern crate diesel;

use serde::{Deserialize, Serialize};

mod config;
pub use config::EntropyConfig;

pub mod db;
pub mod poacher;
pub mod storage;
pub mod web;

#[macro_use]
extern crate rocket;

// Ideally we should make these variants enforce valid values for lat and lng,
// but since Coordinates aren't used for anything but passing to meetup API, I
// am keeping them as loose f32s
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Coordinates {
    lat: f32,
    lng: f32,
}

impl Coordinates {
    pub fn new(lat: f32, lng: f32) -> Self {
        Coordinates { lat, lng }
    }
}
