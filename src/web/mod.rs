use std::path::Path;

use rocket::http::ContentType;
use rocket::response::Debug;
use rocket::{
    figment::{providers::Env, Figment, Profile},
    fs::FileServer,
    Build, Rocket,
};
use rocket_dyn_templates::Template;
use rocket_sync_db_pools::{database, diesel};
use rsass::{compile_scss_path, output};

mod home;
mod events;

#[database("entropy_db")]
pub struct EntropyDbConn(diesel::SqliteConnection);

#[derive(Debug)]
pub enum EntropyWebError {
    DbError(diesel::result::Error),
    ScssError(rsass::Error),
    UnexpectedError(String)
}

pub type EntropyWebResult<T> = Result<T, Debug<EntropyWebError>>;

#[get("/<file>")]
async fn css(file: String) -> EntropyWebResult<(ContentType, String)> {
    let css_base_path = Path::new("src/web/static/scss");
    let path = css_base_path.join(Path::new(&format!("{}.scss", file)));
    let path = path.as_path();

    let format = output::Format {
        style: output::Style::Introspection,
        ..Default::default()
    };
    let css = compile_scss_path(path, format).map_err(|e| EntropyWebError::ScssError(e))?;
    let css = String::from_utf8(css).map_err(|e| EntropyWebError::UnexpectedError(format!("Failed to convert css to string: {}", e)))?;

    Ok((ContentType::CSS, css))
}


fn app() -> Rocket<Build> {
    let figment = Figment::from(rocket::Config::default())
        .merge(("template_dir", "src/web/templates"))
        .merge(("databases.entropy_db.url", "entropy.sqlite3"))
        .merge(Env::prefixed("ENTROPY_").global())
        .select(Profile::from_env_or("ENTROPY_PROFILE", "default"));

    rocket::custom(figment)
        .mount("/", home::routes())
        .mount("/events", events::routes())
        .mount("/css", routes![css])
        .mount("/", FileServer::from("src/web/static"))
        .attach(Template::fairing())
        .attach(EntropyDbConn::fairing())
}

pub async fn run() -> () {
    app().launch().await.unwrap();
}
