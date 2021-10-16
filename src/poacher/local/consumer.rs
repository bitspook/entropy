use diesel::{insert_into, PgConnection, RunQueryDsl};

use crate::db::models::{Group, NewEvent, NewGroup};

use super::{
    models::{LocalEvent, LocalGroup},
    LocalResult,
};

pub async fn process_poached_event(event: LocalEvent, conn: &PgConnection) {
    use crate::db::schema::events::dsl::*;

    let event_group = Group::with_slug(&event.group_slug, &conn);
    if let Err(err) = event_group {
        error!(
            "Failed to find group for event [local_event={:#?}, err={}]",
            event, err
        );

        return;
    }
    let event_group = event_group.unwrap();
    let mut new_event: NewEvent = event.into();
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

pub async fn process_poached_group(group: LocalGroup, conn: &PgConnection) {
    use crate::db::schema::groups::dsl::*;

    let new_group: NewGroup = group.into();
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

pub async fn consume(result: LocalResult, conn: &PgConnection) {
    match result {
        LocalResult::Event(event) => process_poached_event(event, conn).await,
        LocalResult::Group(group) => process_poached_group(group, conn).await,
    }
}
