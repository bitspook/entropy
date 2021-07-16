/// Top level utility functions I don't know where to put yet
use diesel::{replace_into, RunQueryDsl, SqliteConnection};
use entropy::{Coordinates, Meetup, MeetupEvent, MeetupGroup, PoacherMessage};
use log::{debug, error};
use std::sync::Arc;
use tokio::{self, sync::mpsc::Sender};

pub async fn process_scraped_meetup_group(group: MeetupGroup, conn: &SqliteConnection) {
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

pub async fn process_scraped_meetup_event(event: MeetupEvent, conn: &SqliteConnection) {
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

pub async fn search_groups_of_chandigarh(meetup: Arc<Meetup>, tx: Sender<PoacherMessage>) {
    // Meetup's search is trash. A lot of meetup groups get left out when searching by location because
    // Searching for following queries give better results for meetup groups of city
    // apparently all the search terms can be given in a single query, seperated by ", "
    let search_terms = vec![
        "chandigarh",
        "tricity",
        "mohali",
        "punjab",
        "hack",
        "security",
    ];
    let chd_coords = Arc::new(Coordinates::new(30.75, 76.78));
    let radius = 100;

    for term in search_terms.iter().map(|s| s.to_owned()) {
        let meetup = meetup.clone();
        let chd_coords = chd_coords.clone();
        let tx = tx.clone();

        tokio::spawn(async move {
            if let Err(err) = meetup
                .as_ref()
                .search_groups(&chd_coords, term, radius)
                .await
            {
                tx.send(PoacherMessage::Error(err)).await.unwrap();
            };
        });
    }
}
