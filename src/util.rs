/// Top level utility functions I don't know where to put yet
use diesel::{replace_into, RunQueryDsl, SqliteConnection};
use entropy::{EntropyConfig, Meetup, MeetupEvent, MeetupGroup, PoacherMessage};
use log::{debug, error};
use std::sync::Arc;
use tokio::{self, sync::mpsc::Sender};

pub async fn process_scraped_meetup_group(group: MeetupGroup, conn: &SqliteConnection) {
    use entropy::db::schema::meetup_groups::dsl::*;

    let query = replace_into(meetup_groups).values(&group);

    if let Err(err) = query.execute(conn) {
        error!(
            "Failed to insert group \"{}({})\" in db: {:#?}",
            group.name, group.id, err
        );

        return;
    }

    debug!("Saved group in database: {}", group.name);
}

pub async fn process_scraped_meetup_event(event: MeetupEvent, conn: &SqliteConnection) {
    use entropy::db::schema::meetup_events::dsl::*;

    let query = replace_into(meetup_events).values(&event);

    if let Err(err) = query.execute(conn) {
        error!(
            "Failed to insert event \"{}({})\" in db: {:#?}",
            event.title, event.id, err
        );

        return;
    }

    debug!("Saved event in database: {}", event.title);
}

pub async fn search_groups(meetup: Arc<Meetup>, tx: Sender<PoacherMessage>) {
    let config = EntropyConfig::load().unwrap();
    let config = config.poacher.meetup_com;

    for config in config.into_iter() {
        let search_terms = config.search_terms;
        let chd_coords = Arc::new(config.coordinates);
        let radius = config.radius;

        for term in search_terms.iter().map(|s| s.to_owned()) {
            let meetup = meetup.clone();
            let chd_coords = chd_coords.clone();
            let tx = tx.clone();

            tokio::spawn(async move {
                if let Err(err) = meetup
                    .as_ref()
                    .search_groups(&chd_coords, &term, radius)
                    .await
                {
                    tx.send(PoacherMessage::Error(err)).await.unwrap();
                };
            });
        }
    }
}

pub async fn search_events(meetup: Arc<Meetup>, tx: Sender<PoacherMessage>) {
    let config = EntropyConfig::load().unwrap();
    let config = config.poacher.meetup_com;

    for config in config.into_iter() {
        let coords = Arc::new(config.coordinates);
        let radius = config.radius;

        let coords = coords.clone();
        let tx = tx.clone();
        let meetup = meetup.clone();

        tokio::spawn(async move {
            if let Err(err) = meetup.search_events(&coords, radius).await {
                tx.send(PoacherMessage::Error(err)).await.unwrap();
            };
        });
    }
}
