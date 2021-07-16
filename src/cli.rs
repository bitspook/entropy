use diesel::SqliteConnection;
use entropy::web;
use entropy::{Meetup, MeetupResult, PoachedResult, PoacherMessage};
use std::sync::Arc;
use structopt::StructOpt;
use tokio::{
    self,
    sync::mpsc::{self, Receiver, Sender},
};

use crate::util::{
    process_scraped_meetup_event, process_scraped_meetup_group, search_groups_of_chandigarh,
};

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
        #[structopt(long = "group-slug")]
        group_slug: String,
        #[structopt(long = "group-id")]
        // Group-id requirement is silly and only exists because I needed
        // group-id to maintain a foreign key against every meetup-event. We can
        // get rid of this by making group-slug the key for every group, and use
        // that as a FK. I am leaving this here for now since this command will
        // probably not be used much, and I need to keep my momentum. Creating a
        // task (#7) to make group-slug the FK instead of group id and make this
        // option simpler.
        group_id: String,
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

pub async fn run(conn: &SqliteConnection, cmd: CliCmd) -> Result<(), &'static str> {
    let user_agent = "Mozilla/5.0 (X11; Linux x86_64; rv:88.0) Gecko/20100101 Firefox/88.0";

    match cmd {
        CliCmd::Poach(poach_opts) => {
            let client = reqwest::Client::builder()
                .user_agent(user_agent)
                .build()
                .unwrap();
            let (tx, mut rx): (Sender<PoacherMessage>, Receiver<PoacherMessage>) =
                mpsc::channel(1024);

            match poach_opts {
                PoachCmd::Meetup(poach_meetup_opts) => {
                    let meetup = Arc::new(Meetup::new(client.clone(), tx.clone()));

                    match poach_meetup_opts {
                        MeetupCmd::Groups { city, .. } => {
                            let meetup = meetup.clone();

                            if let Some(city) = city {
                                if city.to_lowercase() == "chandigarh" {
                                    tokio::spawn(async move {
                                        search_groups_of_chandigarh(meetup, tx).await;
                                    });
                                } else {
                                    error!("Unsupported city: '{}'.\nPlease use --lat, --lng, --query, --radius to target unsupported cities", city);

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
                        MeetupCmd::Events {
                            group_slug,
                            group_id,
                        } => {
                            let meetup = meetup.clone();
                            tokio::spawn(async move {
                                meetup.fetch_group_events(group_slug, group_id).await;
                            });
                        }
                    }

                    // This handler loop don't belong here. It should be one
                    // layer up, in poach command's handler. I need to figure
                    // out a way to re-initiate work on received messages. e.g
                    // when I receive a meetup-group, I want to initiate more
                    // work using the `meetup` poacher using received data.
                    // Since for now we only have a single poacher and having
                    // access to its reference is really convenient, I am
                    // keeping this code like this.
                    while let Some(msg) = rx.recv().await {
                        match msg {
                            PoacherMessage::Error(err) => {
                                error!("Encountered error when searching groups: {:#?}", err)
                            }
                            PoacherMessage::ResultItem(item) => match item {
                                PoachedResult::Meetup(result) => match result {
                                    MeetupResult::Group(group) => {
                                        let slug = group.slug();
                                        let group_id = group.id.clone();
                                        process_scraped_meetup_group(group, &conn).await;

                                        let meetup = meetup.clone();
                                        tokio::spawn(async move {
                                            meetup.fetch_group_events(slug, group_id).await;
                                        });
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
            };
        }
        CliCmd::Web => web::run().await,
    };

    Ok(())
}
