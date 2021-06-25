#[macro_use]
extern crate diesel;

#[macro_use]
extern crate diesel_migrations;

use diesel::{prelude::*, replace_into};
use entropy::{
    Coordinates, Meetup, MeetupEvent, MeetupGroup, MeetupResult, PoachedResult, PoacherMessage,
};
use env_logger::Env;
use log::{debug, error, warn};
use reqwest;
use std::{io::Error, process::{exit}, sync::Arc};
use structopt::StructOpt;
use tokio::{
    self,
    sync::mpsc::{self, Receiver, Sender},
};

mod cli;
mod db;

embed_migrations!();

fn mk_logger(verbosity: i32) {
    let level = match verbosity {
        0 => "info",
        1 => "debug",
        _ => "trace",
    };

    let env = Env::default()
        .filter_or("ENTROPY_LOG_LEVEL", level)
        .write_style_or("ENTROPY_LOG_STYLE", "always");

    env_logger::init_from_env(env);
}

#[tokio::main]
async fn main() -> Result<(), &'static str> {
    let cli_opts = cli::CliOpts::from_args();

    mk_logger(cli_opts.verbosity);

    debug!("OPTIONS: {:#?}", cli_opts);

    let user_agent = "Mozilla/5.0 (X11; Linux x86_64; rv:88.0) Gecko/20100101 Firefox/88.0";

    let db_con = db::establish_connection();
    embedded_migrations::run(&db_con).expect("Failed to run db migrations");

    match cli_opts.cmd {
        cli::CliCmd::Poach(poach_opts) => {
            let client = reqwest::Client::builder()
                .user_agent(user_agent)
                .build()
                .unwrap();
            let (tx, mut rx): (Sender<PoacherMessage>, Receiver<PoacherMessage>) =
                mpsc::channel(1024);

            match poach_opts {
                cli::PoachCmd::Meetup(poach_meetup_opts) => {
                    let meetup = Arc::new(Meetup::new(client.clone(), tx.clone()));

                    match poach_meetup_opts {
                        cli::MeetupCmd::Groups { city, .. } => {
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
                        cli::MeetupCmd::Events {
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
                                        process_scraped_meetup_group(group, &db_con).await;

                                        let meetup = meetup.clone();
                                        tokio::spawn(async move {
                                            meetup.fetch_group_events(slug, group_id).await;
                                        });
                                    }
                                    MeetupResult::Event(event) => {
                                        process_scraped_meetup_event(event, &db_con).await;
                                    }
                                },
                            },
                            PoacherMessage::Warning(w) => {
                                warn!("Encountered warning: {:#?}", w)
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

async fn process_scraped_meetup_group(group: MeetupGroup, conn: &SqliteConnection) {
    use entropy::db::schema::meetup_groups::dsl::*;
    let new_group = group.to_db_insertable();

    let query = replace_into(meetup_groups).values(&new_group);

    if let Err(err) = query.execute(conn) {
        error!(
            "Failed to insert group \"{}({})\" in db: {:#?}",
            new_group.name, new_group.id, err
        );

        return;
    }

    debug!("Saved group in database: {}", new_group.name);
}

async fn process_scraped_meetup_event(event: MeetupEvent, conn: &SqliteConnection) {
    use entropy::db::schema::meetup_events::dsl::*;
    let new_event = event.to_db_insertable();

    let query = replace_into(meetup_events).values(&new_event);

    if let Err(err) = query.execute(conn) {
        error!(
            "Failed to insert event \"{}({})\" in db: {:#?}",
            new_event.name, new_event.id, err
        );

        return;
    }

    debug!("Saved event in database: {}", new_event.name);
}

async fn search_groups_of_chandigarh(meetup: Arc<Meetup>, tx: Sender<PoacherMessage>) {
    // Meetup's search is trash. A lot of meetup groups get left out when searching by location because
    // Searching for following queries give better results for meetup groups of city
    // apparently all the search terms can be given in a single query, seperated by ", "
    let search_terms = vec!["chandigarh", "tricity", "mohali", "punjab"];
    let chd_coords = Arc::new(Coordinates::new(30.75, 76.78));
    let radius = 100;

    for term in search_terms.iter().map(|s| s.to_owned()) {
        let meetup = meetup.clone();
        let chd_coords = chd_coords.clone();
        let tx = tx.clone();

        tokio::spawn(async move {
            if let Err(err) = meetup.as_ref().search_groups(&chd_coords, term, radius).await {
                tx.send(PoacherMessage::Error(err)).await.unwrap();
            };
        });
    }
}
