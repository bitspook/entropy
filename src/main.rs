#[macro_use]
extern crate diesel;

#[macro_use]
extern crate diesel_migrations;

use diesel::{prelude::*, replace_into};
use entropy::{Coordinates, Meetup, MeetupEvent, MeetupGroup, MeetupResult, ScraperMessage, ScraperResult};
use env_logger::Env;
use log::{debug, error, warn};
use reqwest;
use std::sync::Arc;
use tokio::{
    self,
    sync::mpsc::{self, Receiver, Sender},
};

mod db;

embed_migrations!();

fn mk_logger() {
    let env = Env::default()
        .filter_or("ENTROPY_LOG_LEVEL", "debug")
        .write_style_or("ENTROPY_LOG_STYLE", "always");

    env_logger::init_from_env(env);
}

#[tokio::main]
async fn main() {
    mk_logger();

    let user_agent = "Mozilla/5.0 (X11; Linux x86_64; rv:88.0) Gecko/20100101 Firefox/88.0";

    let db_con = db::establish_connection();
    embedded_migrations::run(&db_con).expect("Failed to run db migrations");

    let client = reqwest::Client::builder()
        .user_agent(user_agent)
        .build()
        .unwrap();

    let (tx, mut rx): (Sender<ScraperMessage>, Receiver<ScraperMessage>) = mpsc::channel(1024);

    let meetup = Arc::new(Meetup::new(client.clone(), tx.clone()));

    let meetup2 = meetup.clone();
    tokio::spawn(async move {
        search_groups_of_chandigarh(meetup2, tx).await;
    });

    while let Some(msg) = rx.recv().await {
        match msg {
            ScraperMessage::Error(err) => {
                error!("Encountered error when searching groups: {:#?}", err)
            }
            ScraperMessage::ResultItem(item) => match item {
                ScraperResult::Meetup(result) => match result {
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
            ScraperMessage::Warning(w) => {
                warn!("Encountered warning: {:#?}", w)
            }
        }
    }
}

async fn process_scraped_meetup_group(group: MeetupGroup, conn: &SqliteConnection) {
    use entropy::db::schema::meetup_groups::dsl::*;
    let new_group = group.to_db_insertable();

    let query = replace_into(meetup_groups).values(&new_group);

    // let debug = debug_query::<diesel::sqlite::Sqlite, _>(&query);
    // debug!("Making query: {}", debug);

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

    // let debug = debug_query::<diesel::sqlite::Sqlite, _>(&query);
    // debug!("Making query: {}", debug);

    if let Err(err) = query.execute(conn) {
        error!(
            "Failed to insert event \"{}({})\" in db: {:#?}",
            new_event.name, new_event.id, err
        );

        return;
    }

    debug!("Saved event in database: {}", new_event.name);
}

async fn search_groups_of_chandigarh(meetup: Arc<Meetup>, tx: Sender<ScraperMessage>) {
    // Meetup's search is trash. A lot of meetup groups get left out when searching by location because
    // Searching for following queries give better results for meetup groups of city
    // apparently all the search terms can be given in a single query, seperated by ", "
    let search_terms = vec!["chandigarh", "tricity", "mohali", "punjab"];

    let chd_coords = Arc::new(Coordinates::new(30.75, 76.78));

    for term in search_terms.iter().map(|s| s.to_owned()) {
        let meetup = meetup.clone();
        let chd_coords = chd_coords.clone();
        let tx = tx.clone();

        tokio::spawn(async move {
            if let Err(err) = meetup.as_ref().search_groups(&chd_coords, term).await {
                tx.send(ScraperMessage::Error(err)).await.unwrap();
            };
        });
    }
}
