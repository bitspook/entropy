use rocket::{
    figment::{providers::Env, Figment, Profile},
    Build, Rocket,
};

#[get("/")]
fn hello() -> &'static str {
    "Hello world"
}

fn app() -> Rocket<Build> {
    let figment = Figment::from(rocket::Config::default())
        .merge(Env::prefixed("ENTROPY_").global())
        .select(Profile::from_env_or("ENTROPY_PROFILE", "default"));

    rocket::custom(figment).mount("/", routes![hello])
}

pub async fn run() -> () {
    app().launch().await.unwrap();
}
