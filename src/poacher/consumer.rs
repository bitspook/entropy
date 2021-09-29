use diesel::{insert_into, PgConnection, RunQueryDsl};
use log::{debug, error};
use tokio::sync::mpsc::Receiver;

use crate::db;

use super::{
    meetup::{MeetupEvent, MeetupGroup, MeetupResult},
    PoacherMessage, PoacherResult,
};

pub async fn process_scraped_meetup_group(
    group: MeetupGroup,
    conn: &PgConnection,
    groups_blacklist: &Vec<String>,
) {
    use crate::db::schema::meetup_groups::dsl::*;

    if groups_blacklist.contains(&group.slug) {
        warn!("Ignoring blacklisted group: {}", group.slug);
        return;
    }

    let query = insert_into(meetup_groups).values(&group);

    if let Err(err) = query.execute(conn) {
        error!(
            "Failed to insert group \"{}({})\" in db: {:#?}",
            group.name, group.id, err
        );

        return;
    }

    debug!("Saved group in database: {}", group.name);
}

pub async fn process_scraped_meetup_event(
    event: MeetupEvent,
    conn: &PgConnection,
    groups_blacklist: &Vec<String>,
) {
    use crate::db::schema::meetup_events::dsl::*;

    if groups_blacklist.contains(&event.group_slug) {
        warn!("Ignoring event for blacklisted group: {}", event.group_slug);
        return;
    }

    let query = insert_into(meetup_events).values(&event);

    if let Err(err) = query.execute(conn) {
        error!(
            "Failed to insert event \"{}({})\" in db: {:#?}",
            event.title, event.id, err
        );

        return;
    }

    debug!("Saved event in database: {}", event.title);
}

/// Absorb all the poacher messages from `rx` and spawn tasks to process them.
pub async fn run(
    mut rx: Receiver<PoacherMessage>,
    groups_blacklist: &Vec<String>,
) -> anyhow::Result<()> {
    let conn = db::establish_connection()?;
    while let Some(msg) = rx.recv().await {
        match msg {
            PoacherMessage::Error(err) => {
                error!("Encountered error when poaching: {:#?}", err)
            }
            PoacherMessage::ResultItem(item) => match item {
                PoacherResult::Meetup(result) => match result {
                    MeetupResult::Group(group) => {
                        process_scraped_meetup_group(group, &conn, groups_blacklist).await;
                    }
                    MeetupResult::Event(event) => {
                        process_scraped_meetup_event(event, &conn, groups_blacklist).await;
                    }
                },
            },
            PoacherMessage::Warning(w) => {
                warn!("Encountered warning: {:#?}", w)
            }
        }
    }

    Ok(())
}
