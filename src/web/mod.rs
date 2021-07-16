use rocket::{Build, Rocket};

#[get("/")]
fn hello() -> &'static str {
    "Hello world"
}

pub fn app() -> Rocket<Build> {
    rocket::build().mount("/", routes![hello])
}
