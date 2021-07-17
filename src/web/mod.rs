use diesel::prelude::*;
use rocket::response::Debug;
use rocket::serde::json::Json;
use rocket::{
    figment::{providers::Env, Figment, Profile},
    fs::FileServer,
    Build, Rocket,
};
use rocket_dyn_templates::Template;
use rocket_sync_db_pools::{database, diesel};
use serde_json::json;

use crate::db::models::MeetupEvent;

#[database("entropy_db")]
struct EntropyDbConn(diesel::SqliteConnection);

#[get("/")]
fn index() -> Template {
    let context = json!({});

    Template::render("index", context)
}

#[get("/events")]
async fn events(db: EntropyDbConn) -> Result<Json<Vec<MeetupEvent>>, Debug<diesel::result::Error>> {
    use crate::db::schema::meetup_events::dsl::*;

    let events = db
        .run(|conn| meetup_events.limit(5).load::<MeetupEvent>(conn))
        .await?;

    Ok(Json(events))
}

fn app() -> Rocket<Build> {
    let figment = Figment::from(rocket::Config::default())
        .merge(("template_dir", "src/web/templates"))
        .merge(("databases.entropy_db.url", "entropy.sqlite3"))
        .merge(Env::prefixed("ENTROPY_").global())
        .select(Profile::from_env_or("ENTROPY_PROFILE", "default"));

    rocket::custom(figment)
        .mount("/", routes![index, events])
        .mount("/", FileServer::from("src/web/static"))
        .attach(Template::fairing())
        .attach(EntropyDbConn::fairing())
}

pub async fn run(conn: SqliteConnection) -> () {
    drop(conn);

    app().launch().await.unwrap();
}
