use diesel::{insert_into, PgConnection, RunQueryDsl};
use uuid::Uuid;

use crate::db::models::{Group, Initiative, NewEvent, NewEventSection, NewGoal, NewGroup};

use super::{
    models::{LocalEvent, LocalEventSection, LocalGoal, LocalGroup, LocalInitiative},
    LocalResult,
};

pub async fn process_poached_event(
    conn: &PgConnection,
    event: LocalEvent,
    sections: Vec<LocalEventSection>,
) {
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
    let mut new_event = NewEvent::from(event);
    new_event.group_id = Some(event_group.id);
    let query = insert_into(events).values(&new_event);

    match query.returning(id).get_result::<Uuid>(conn) {
        Err(err) => {
            error!(
                "Failed to insert event \"{}({})\" in db: {:#?}",
                new_event.title, new_event.slug, err
            );
        }
        Ok(inserted_event_id) => {
            info!("Saved event in database: {}", new_event.title);
            use crate::db::schema::event_sections::dsl::*;

            let new_event_sections: Vec<NewEventSection> = sections
                .into_iter()
                .map(|e| {
                    let mut section = NewEventSection::from(e);
                    section.event_id = Some(inserted_event_id);

                    section
                })
                .collect();

            let query = insert_into(event_sections).values(&new_event_sections);
            if let Err(err) = query.execute(conn) {
                error!("Failed to insert event-sections: {:?}", err);
            } else {
                debug!("Saved event-sections in database");
            }
        }
    }
}

pub async fn process_poached_group(conn: &PgConnection, group: LocalGroup) {
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

async fn process_poached_initiative(
    conn: &PgConnection,
    initiative: LocalInitiative,
    goals_vec: Vec<LocalGoal>,
) {
    use crate::db::schema::initiatives::dsl::*;

    let new_initiative = Initiative::from(initiative);
    let query = insert_into(initiatives).values(&new_initiative);

    // TODO: We don't need to wait for initiative to get inserted. We already
    // have the slug here. Dum dum!
    match query.returning(slug).get_result::<String>(conn) {
        Err(err) => {
            error!(
                "Failed to insert initiative \"{}({})\" in db: {:#?}",
                new_initiative.title, new_initiative.slug, err
            );
        }
        Ok(inserted_initiative_slug) => {
            info!("Saved initiative in database: {}", new_initiative.title);
            use crate::db::schema::goals::dsl::*;

            let new_goals: Vec<NewGoal> = goals_vec
                .into_iter()
                .map(|g| {
                    let mut goal = NewGoal::from(g);
                    goal.initiative_slug = Some(inserted_initiative_slug.clone());

                    goal
                })
                .collect();

            let query = insert_into(goals).values(&new_goals);
            if let Err(err) = query.execute(conn) {
                error!("Failed to insert goals: {:?}", err);
            } else {
                debug!("Saved goals in database");
            }
        }
    }
}

pub async fn consume(result: LocalResult, conn: &PgConnection) {
    match result {
        LocalResult::Event(event, sections) => process_poached_event(conn, event, sections).await,
        LocalResult::Group(group) => process_poached_group(conn, group).await,
        LocalResult::Initiative(initiative, goals) => {
            process_poached_initiative(conn, initiative, goals).await
        }
    }
}
