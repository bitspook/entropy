use std::collections::HashMap;

use rocket::{Build, Rocket, figment::{providers::Env, Figment, Profile}, fs::FileServer};
use rocket_dyn_templates::Template;

#[get("/")]
fn index() -> Template {
    let context: HashMap<String, String> = HashMap::new();

    Template::render("index", context)
}

fn app() -> Rocket<Build> {
    let figment = Figment::from(rocket::Config::default())
        .merge(("template_dir", "src/web/templates"))
        .merge(Env::prefixed("ENTROPY_").global())
        .select(Profile::from_env_or("ENTROPY_PROFILE", "default"));

    rocket::custom(figment)
        .mount("/", routes![index])
        .mount("/", FileServer::from("src/web/static"))
        .attach(Template::fairing())
}

pub async fn run() -> () {
    app().launch().await.unwrap();
}
