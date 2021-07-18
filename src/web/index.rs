use diesel::prelude::*;
use rocket::Route;
use rocket_dyn_templates::Template;
use rocket_sync_db_pools::diesel;
use serde_json::json;

use crate::MeetupEvent;

use super::{DbError, EntropyDbConn};

#[get("/")]
async fn index(db: EntropyDbConn) -> Result<Template, DbError> {
    use crate::db::schema::meetup_events::dsl::*;

    let events = db
        .run(|conn| {
            meetup_events
                .order(start_time.desc())
                .limit(5)
                .load::<MeetupEvent>(conn)
        })
        .await?;

    let context = json!({ "events": events });

    Ok(Template::render("index", context))
}

pub fn routes() -> Vec<Route> {
    routes![index]
}
