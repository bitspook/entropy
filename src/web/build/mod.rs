use super::dev::events::build as build_events_list;
use super::dev::home::build as build_home;
use anyhow::{bail, Context, Error, Result};
use fs_extra::dir::{copy, get_dir_content, CopyOptions};
use rsass::{compile_scss_path, output};
use std::{
    fs::{self, DirBuilder},
    io,
    path::Path,
};

pub async fn build() {
    info!("Building public static website");

    let dist_dir = "dist";
    let static_dir = "src/web/static";

    debug!("Creating dist directory");
    if let Err(err) = DirBuilder::new().create(dist_dir) {
        match err.kind() {
            io::ErrorKind::AlreadyExists => {
                warn!("dist already exists. Will overwrite colliding files.")
            }
            _ => {
                error!("Failed to create dist dir: {:#}", err);
                return;
            }
        }
    };

    debug!("Copying content of static dir to dist");
    let mut copy_opts = CopyOptions::new();
    copy_opts.copy_inside = true;
    copy_opts.content_only = true;
    copy_opts.overwrite = true;

    if let Err(err) = copy(static_dir, dist_dir, &copy_opts) {
        error!("Error while copying static dir to dist: {:#}", err);
        return;
    }

    debug!("Building SCSS");
    if let Err(err) = build_scss().await {
        error!("Failed to build SCSS: {:#}", err);
    }

    debug!("Building HTML");
    if let Err(err) = build_html().await {
        error!("Failed to build HTML: {:#}", err);
    }
}

async fn build_scss() -> Result<()> {
    let scss_dir = "src/web/scss";
    let css_dir = "dist/css";
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

async fn build_html() -> Result<()> {
    let rocket = crate::web::dev::app();
    let client = rocket::local::asynchronous::Client::untracked(rocket).await?;
    let dist_path = Path::new("dist");

    debug!("Building home page");
    build_home(&client, dist_path).await?;

    debug!("Building events list page");
    build_events_list(&client, dist_path).await?;

    Ok(())
}
