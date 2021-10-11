use diesel::{insert_into, PgConnection, RunQueryDsl};

use crate::db::models::{Group, NewEvent, NewGroup};

use super::{MeetupEvent, MeetupGroup, MeetupResult};

pub async fn process_scraped_meetup_group(
    meetup_group: MeetupGroup,
    conn: &PgConnection,
    groups_blacklist: &Vec<String>,
) {
    use crate::db::schema::groups::dsl::*;

    if groups_blacklist.contains(&meetup_group.slug) {
        warn!("Ignoring blacklisted group: {}", meetup_group.slug);
        return;
    }

    let new_group: NewGroup = meetup_group.into();
    let query = insert_into(groups).values(&new_group);

    if let Err(err) = query.execute(conn) {
        error!(
            "Failed to insert group \"{}({})\" in db: {:#?}",
            new_group.name, new_group.slug, err
        );

        return;
    }

    debug!("Saved group in database: {}", new_group.name);
}

pub async fn process_scraped_meetup_event(
    meetup_event: MeetupEvent,
    conn: &PgConnection,
    groups_blacklist: &Vec<String>,
) {
    use crate::db::schema::events::dsl::*;

    if groups_blacklist.contains(&meetup_event.group_slug) {
        warn!(
            "Ignoring event for blacklisted group: {}",
            meetup_event.group_slug
        );
        return;
    }

    let event_group = Group::with_slug(&meetup_event.group_slug, &conn);
    if let Err(err) = event_group {
        error!(
            "Failed to find group for event [meetup_event={:#?}, err={}]",
            meetup_event, err
        );

        return;
    }
    let event_group = event_group.unwrap();
    let mut new_event: NewEvent = meetup_event.into();
    new_event.group_id = Some(event_group.id);
    let query = insert_into(events).values(&new_event);

    if let Err(err) = query.execute(conn) {
        error!(
            "Failed to insert event \"{}({})\" in db: {:#?}",
            new_event.title, new_event.slug, err
        );

        return;
    }

    debug!("Saved event in database: {}", new_event.title);
}

pub async fn consume(
    result: MeetupResult,
    conn: &diesel::PgConnection,
    groups_blacklist: &Vec<String>,
) {
    match result {
        MeetupResult::Group(group) => {
            process_scraped_meetup_group(group, &conn, groups_blacklist).await;
        }
        MeetupResult::Event(event) => {
            process_scraped_meetup_event(event, &conn, groups_blacklist).await;
        }
    }
}
