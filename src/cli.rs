use entropy::web;
use entropy::{Meetup, MeetupResult, PoachedResult, PoacherMessage};
use std::sync::Arc;
use structopt::StructOpt;
use tokio::{
    self,
    sync::mpsc::{self, Receiver, Sender},
};

use crate::db;
use crate::util::{process_scraped_meetup_event, process_scraped_meetup_group, search_events_of_chandigarh, search_groups_of_chandigarh};

#[derive(StructOpt, Debug)]
pub enum MeetupCmd {
    Groups {
        #[structopt(long = "with-events")]
        with_events: bool,
        #[structopt(long, conflicts_with_all(&["lng", "lat"]))]
        /// One of the supported cities. lat, lng, radius and query for supported
        /// cities are already known to entropy and don't need to be provided
        city: Option<String>,
        #[structopt(long, required_unless("city"), requires("lng"))]
        /// Latitude of place around which groups/events should be found
        lat: Option<f32>,
        #[structopt(long, required_unless("city"), requires("lat"))]
        /// Longitude of place around which groups/events should be found
        lng: Option<f32>,
        #[structopt(long, conflicts_with("city"))]
        /// Radius in miles
        radius: Option<u32>,
        #[structopt(short, long, conflicts_with("city"))]
        /// Space separated list of queries you want to search the groups by
        query: Option<Vec<String>>,
    },
    Events {
        #[structopt(long)]
        /// One of the supported cities. lat, lng, radius and query for supported
        /// cities are already known to entropy and don't need to be provided
        city: Option<String>,
    },
}

#[derive(StructOpt, Debug)]
pub enum PoachCmd {
    Meetup(MeetupCmd),
}

#[derive(StructOpt, Debug)]
pub enum CliCmd {
    /// Manage scrappers for aggregating content from web
    Poach(PoachCmd),
    /// Manage entropy web apps
    Web,
}

#[derive(StructOpt, Debug)]
#[structopt(name = "entropy", about = "Manage the entropy website")]
pub struct CliOpts {
    #[structopt(short, global = true, parse(from_occurrences))]
    pub verbosity: i32,

    #[structopt(subcommand)]
    pub cmd: CliCmd,
}

pub async fn run(cmd: CliCmd) -> Result<(), &'static str> {
    match cmd {
        CliCmd::Poach(poach_opts) => {
            let (tx, rx): (Sender<PoacherMessage>, Receiver<PoacherMessage>) = mpsc::channel(1024);
            let user_agent = "Mozilla/5.0 (X11; Linux x86_64; rv:88.0) Gecko/20100101 Firefox/88.0";
            let client = reqwest::Client::builder()
                .user_agent(user_agent)
                .build()
                .unwrap();
            let meetup = Arc::new(Meetup::new(client, tx.clone()));

            match poach_opts {
                PoachCmd::Meetup(poach_meetup_opts) => {
                    match poach_meetup_opts {
                        MeetupCmd::Groups { city, .. } => {
                            if let Some(city) = city {
                                if city.to_lowercase() == "chandigarh" {
                                    search_groups_of_chandigarh(meetup, tx).await;
                                } else {
                                    error!("Unsupported city: '{}'", city);

                                    return Err("Unsupported city");
                                }
                            } else {
                                // TODO: Implement this part. Right now I am not
                                // sure if this will ever be used. I created
                                // these options because I wanted to play with
                                // how CLIs are created in Rust. Created #8 to
                                // add these options if they're ever needed. Or
                                // remove this code.
                                return Err("Not Implemented");
                            }
                        }
                        MeetupCmd::Events { city, .. } => {
                            if let Some(city) = city {
                                if city.to_lowercase() == "chandigarh" {
                                    search_events_of_chandigarh(meetup, tx).await;
                                } else {
                                    error!("Unsupported city: '{}'", city);

                                    return Err("Unsupported city");
                                }
                            } else {
                                // TODO: Implement this part. Right now I am not
                                // sure if this will ever be used. I created
                                // these options because I wanted to play with
                                // how CLIs are created in Rust. Created #8 to
                                // add these options if they're ever needed. Or
                                // remove this code.
                                return Err("Not Implemented");
                            }
                        }
                    }
                }
            };

            poacher_meditation(rx).await;
        }
        CliCmd::Web => web::run().await,
    };

    Ok(())
}

/// Absorb all the poacher messages from `rx` and spawn tasks to process them.
async fn poacher_meditation(mut rx: Receiver<PoacherMessage>) {
    let conn = db::establish_connection();
    while let Some(msg) = rx.recv().await {
        match msg {
            PoacherMessage::Error(err) => {
                error!("Encountered error when poaching: {:#?}", err)
            }
            PoacherMessage::ResultItem(item) => match item {
                PoachedResult::Meetup(result) => match result {
                    MeetupResult::Group(group) => {
                        process_scraped_meetup_group(group, &conn).await;
                    }
                    MeetupResult::Event(event) => {
                        process_scraped_meetup_event(event, &conn).await;
                    }
                },
            },
            PoacherMessage::Warning(w) => {
                warn!("Encountered warning: {:#?}", w)
            }
        }
    }
}
