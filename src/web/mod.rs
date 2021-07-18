use rocket::response::Debug;
use rocket::{
    figment::{providers::Env, Figment, Profile},
    fs::FileServer,
    Build, Rocket,
};
use rocket_dyn_templates::Template;
use rocket_sync_db_pools::{database, diesel};

mod index;

#[database("entropy_db")]
pub struct EntropyDbConn(diesel::SqliteConnection);

type DbError = Debug<diesel::result::Error>;

fn app() -> Rocket<Build> {
    let figment = Figment::from(rocket::Config::default())
        .merge(("template_dir", "src/web/templates"))
        .merge(("databases.entropy_db.url", "entropy.sqlite3"))
        .merge(Env::prefixed("ENTROPY_").global())
        .select(Profile::from_env_or("ENTROPY_PROFILE", "default"));

    rocket::custom(figment)
        .mount("/", index::routes())
        .mount("/", FileServer::from("src/web/static"))
        .attach(Template::fairing())
        .attach(EntropyDbConn::fairing())
}

pub async fn run() -> () {
    app().launch().await.unwrap();
}
