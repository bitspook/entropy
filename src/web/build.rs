use crate::EntropyConfig;

use super::routes::event_details::build as build_event_details;
use super::routes::events::build as build_events_list;
use super::routes::home::build as build_home;
use anyhow::{bail, Context, Error, Result};
use fs_extra::dir::{copy, get_dir_content, CopyOptions};
use kuchiki::traits::*;
use rsass::{compile_scss_path, output};
use std::{
    fs::{self, DirBuilder},
    io,
    path::Path,
};
use urlencoding::decode;
use walkdir::WalkDir;

pub async fn build() -> anyhow::Result<()> {
    info!("Building public static website");

    let config = EntropyConfig::load()?;
    let web_config = &config.web;

    let dist_dir = &web_config.static_site.dist_path;
    let static_dir = &web_config.dev_server.static_dir;
    let scss_dir = &web_config.dev_server.scss_dir;
    let css_dir = Path::new(&dist_dir).join("css");
    let css_dir = css_dir.as_path().to_str().unwrap();

    debug!("Creating dist directory");
    if let Err(err) = DirBuilder::new().create(dist_dir.clone()) {
        match err.kind() {
            io::ErrorKind::AlreadyExists => {
                warn!("dist already exists. Will overwrite colliding files.")
            }
            _ => {
                bail!("Failed to create dist dir: {:#}", err);
            }
        }
    };

    debug!("Copying content of static dir to dist");
    let mut copy_opts = CopyOptions::new();
    copy_opts.copy_inside = true;
    copy_opts.content_only = true;
    copy_opts.overwrite = true;

    if let Err(err) = copy(static_dir, dist_dir.clone(), &copy_opts) {
        bail!("Error while copying static dir to dist: {:#}", err);
    }

    debug!("Building SCSS");
    if let Err(err) = build_scss(&scss_dir, css_dir).await {
        error!("Failed to build SCSS: {:#}", err);
    }

    debug!("Building HTML");
    if let Err(err) = build_html().await {
        error!("Failed to build HTML: {:#}", err);
    }

    debug!("Post Processing HTML");
    if let Err(err) = post_process_html(Path::new(&dist_dir), &config).await {
        error!("Failed during post-processing HTML: [err={:#}]", err);
    }

    info!("Build Successful!!");

    Ok(())
}

async fn build_scss(scss_dir: &str, css_dir: &str) -> Result<()> {
    let scss_dir_contents = get_dir_content(scss_dir)?;

    debug!("Creating dist/css directory");
    if let Err(err) = fs::create_dir(css_dir) {
        match err.kind() {
            io::ErrorKind::AlreadyExists => {
                debug!("CSS dir already exists");
            }
            _ => bail!("Failed to create CSS dir"),
        }
    };

    for scss_file in scss_dir_contents.files {
        let scss_path = Path::new(&scss_file);
        let filename = scss_path
            .file_name()
            .ok_or(Error::msg("Failed to get SCSS File name"))?;

        if filename.to_string_lossy().starts_with("_") {
            debug!("Ignoring SCSS partial file: {}", scss_file);
        } else {
            debug!("Compiling SCSS file: {}", scss_file);
            let format = output::Format {
                style: output::Style::Compressed,
                ..Default::default()
            };
            let css = compile_scss_path(scss_path, format)?;
            let css = String::from_utf8(css)?;

            let css_file = Path::new(css_dir).join(filename);
            let css_file = css_file.with_extension("css");
            let css_file = css_file.as_path();
            debug!("Writing SCSS to CSS file: {}", css_file.display());
            fs::write(css_file, css)
                .with_context(|| format!("Writing CSS File: {}", css_file.display()))?;
        }
    }

    Ok(())
}

async fn post_process_html(dist: &Path, config: &EntropyConfig) -> Result<()> {
    for file in WalkDir::new(dist)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let f_name = file.file_name().to_string_lossy();

        if f_name.ends_with(".html") {
            let html = fs::read_to_string(file.path())?;
            let doc = kuchiki::parse_html().one(html);

            let minio_url = &config.storage.credentials.endpoint;
            let static_dir = &config.web.static_site.dist_path;
            let client = reqwest::Client::builder().build()?;

            for css_match in doc.select("img").unwrap() {
                let mut attrs = css_match.attributes.borrow_mut();
                if let Some(src) = attrs.get_mut("src") {
                    if src.contains(minio_url) {
                        let src_url = url::Url::parse(src)?;
                        let mut img_path = decode(src_url.path())?.to_string();
                        if img_path.starts_with('/') {
                            img_path.remove(0);
                        }
                        let img_path = Path::new(static_dir).join("storage").join(img_path);

                        if !img_path.exists() {
                            debug!("Downloading storage asset: [path={:?}]", img_path);
                            let res = client.get(src.clone()).send().await?;
                            let img_content = res.bytes().await?;

                            if let Some(parent) = img_path.parent() {
                                fs::create_dir_all(parent)?;
                            }

                            fs::write(&img_path, &img_content)?;
                        } else {
                            debug!("Not downloading existing storage asset: [path={:?}]", img_path);
                        }

                        src.clear();
                        if let Some(new_src) = img_path.to_str() {
                            let new_src = new_src.replace(static_dir, "");
                            src.insert_str(0, &new_src);
                        }
                    }
                }
            }

            doc.serialize_to_file(file.path()).unwrap();
        }
    }

    Ok(())
}

async fn build_html() -> Result<()> {
    let rocket = crate::web::server::app();
    let client = rocket::local::asynchronous::Client::untracked(rocket).await?;
    let dist_path = Path::new("dist");

    debug!("Building home page");
    build_home(&client, dist_path).await?;

    debug!("Building events list page");
    build_events_list(&client, dist_path).await?;

    debug!("Building event details pages");
    let a_client = std::sync::Arc::new(client);
    build_event_details(a_client, dist_path).await?;

    Ok(())
}
