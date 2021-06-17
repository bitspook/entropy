#[macro_use]
extern crate diesel;

#[macro_use]
extern crate diesel_migrations;

use diesel::{debug_query, prelude::*, replace_into};
use entropy::{Coordinates, Meetup, MeetupGroup, MeetupResult, ScraperMessage, ScraperResult};
use reqwest;
use std::sync::Arc;
use tokio::{
    self,
    sync::mpsc::{self, Receiver, Sender},
};

mod db;

embed_migrations!();

#[tokio::main]
async fn main() {
    let user_agent = "Mozilla/5.0 (X11; Linux x86_64; rv:88.0) Gecko/20100101 Firefox/88.0";

    let db_con = db::establish_connection();
    embedded_migrations::run(&db_con).expect("Failed to run db migrations");

    let client = reqwest::Client::builder()
        .user_agent(user_agent)
        .build()
        .unwrap();

    let (tx, mut rx): (Sender<ScraperMessage>, Receiver<ScraperMessage>) = mpsc::channel(1024);

    let meetup = Meetup::new(client.clone(), tx.clone());
    let meetup = Arc::new(meetup);

    let handle = tokio::spawn(async move {
        search_groups_of_chandigarh(meetup, tx).await;
    });

    while let Some(msg) = rx.recv().await {
        match msg {
            ScraperMessage::Error(err) => {
                println!("Encountered error when searching groups: {:#?}", err)
            }
            ScraperMessage::ResultItem(item) => match item {
                ScraperResult::Meetup(result) => match result {
                    MeetupResult::Group(group) => {
                        process_scraped_meetup_group(group, &db_con).await
                    }
                    _ => {
                        println!("Don't know how to process scraped item: {:#?}", result);
                    }
                },
            },
            ScraperMessage::Warning(w) => {
                println!("Encountered warning: {:#?}", w)
            }
        }
    }

    handle.await.unwrap();
}

async fn process_scraped_meetup_group(group: MeetupGroup, conn: &SqliteConnection) {
    use entropy::db::schema::meetup_groups::dsl::*;
    let new_group = group.to_db_insertable();

    let query = replace_into(meetup_groups).values(&new_group);

    // let debug = debug_query::<diesel::sqlite::Sqlite, _>(&query);
    // println!("Making query: {}", debug);

    if let Err(err) = query.execute(conn) {
        println!(
            "Failed to insert group \"{}({})\" in db: {:#?}",
            new_group.name, new_group.id, err
        );
    };
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
