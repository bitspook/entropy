use std::path::Path;

use anyhow::Error;
use rocket::http::ContentType;
use rocket::{figment::Figment, fs::FileServer, Build, Rocket};
use rocket_dyn_templates::Template;
use rsass::{compile_scss_path, output};

use crate::web::utils::render_md_tera_filter;
use crate::web::{routes, Db, WebResult};
use crate::EntropyConfig;

#[get("/<file>")]
async fn css(file: String) -> WebResult<(ContentType, String)> {
    let scss_dir = Path::new("src/web/scss");
    let path = scss_dir.join(Path::new(&file));
    let path = path.with_extension("scss");
    let path = path.as_path();
    let format = output::Format {
        style: output::Style::Introspection,
        ..Default::default()
    };

    let css = compile_scss_path(path, format).map_err(Error::from)?;
    let css = String::from_utf8(css).map_err(Error::from)?;

    Ok((ContentType::CSS, css))
}

pub fn app() -> Rocket<Build> {
    let config = EntropyConfig::load().expect("Invlaid Configuration");

    let figment = Figment::from(rocket::Config::default())
        .merge(("port", config.server.port))
        .merge(("address", config.server.host))
        .merge(("template_dir", config.server.template_dir))
        .merge(("databases.entropy_db.url", config.database_url));

    rocket::custom(figment)
        .mount("/", routes::home::routes())
        .mount("/", routes::events::routes())
        .mount("/", routes::event_details::routes())
        .mount("/css", routes![css])
        .mount("/", FileServer::from(config.server.static_dir))
        .attach(Template::custom(|engines| {
            engines
                .tera
                .register_filter("render_md", render_md_tera_filter)
        }))
        .attach(Db::fairing())
}

pub async fn run() -> () {
    app().launch().await.unwrap();
}
