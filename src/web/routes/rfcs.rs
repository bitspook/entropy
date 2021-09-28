use std::path::{Path, PathBuf};

use anyhow::{Error, Result};
use futures::TryFutureExt;
use pulldown_cmark::{html, Options, Parser};
use rocket::Route;
use rocket_dyn_templates::Template;
use serde::Serialize;
use serde_json::json;
use tokio::fs::{read, read_dir, DirEntry, ReadDir};

use crate::EntropyConfig;

use crate::web::WebResult;

fn render_md(md: &str) -> String {
    let options = Options::empty();
    let parser = Parser::new_ext(md, options);

    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);

    html_output
}

#[derive(Serialize)]
struct RfcFile {
    name: String,
    slug: String,
    path: PathBuf,
}

impl From<DirEntry> for RfcFile {
    fn from(dir: DirEntry) -> Self {
        let slug = dir.file_name().into_string().unwrap();

        Self {
            name: dir.file_name().into_string().unwrap(),
            slug,
            path: dir.path(),
        }
    }
}

async fn collect_md_files<T: From<DirEntry>>(mut dir: ReadDir) -> Result<Vec<T>> {
    let mut files: Vec<T> = vec![];

    while let Some(file) = dir.next_entry().await.map_err(Error::from)? {
        if !(file.path().extension().unwrap() == "md"
            || file.path().extension().unwrap() == "markdown")
        {
            warn!("Ignoring non markdown file: {:?}", file.path());
            continue;
        }

        files.push(file.into());
    }

    Ok(files)
}

#[get("/rfcs")]
async fn rfcs() -> WebResult<Template> {
    let config = EntropyConfig::load()?;
    let base_url = config.static_site.base_url;
    let rfc_dir = Path::new(&config.rfc_dir);
    let readme_path = rfc_dir.join("readme.md");

    debug!("Reading RFC readme.md: {:?}", readme_path);
    let readme_content = read(readme_path).await.map_err(Error::from)?;
    let readme_content = String::from_utf8(readme_content).map_err(Error::from)?;
    let intro = render_md(&readme_content);

    debug!("Reading all files in rfcs dir");
    let rfc_files = read_dir(rfc_dir).map_err(Error::from).await?;
    let rfc_files: Vec<RfcFile> = collect_md_files(rfc_files).await?;

    let context = json!({ "base_url": base_url, "intro": intro, "rfcs": rfc_files });

    Ok(Template::render("rfcs", context))
}

pub fn routes() -> Vec<Route> {
    routes![rfcs]
}
