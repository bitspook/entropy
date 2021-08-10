use fs_extra::dir::{copy, CopyOptions};
use std::{fs::DirBuilder, io};

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
    } else {
        debug!("Successfully created dist dir");
    };

    debug!("Copying content of static dir to dist");
    let mut copy_opts = CopyOptions::new();
    copy_opts.copy_inside = true;
    copy_opts.content_only = true;
    copy_opts.overwrite = true;

    if let Err(err) = copy(static_dir, dist_dir, &copy_opts) {
        error!("Error while copying static dir to dist: {:#}", err);
        return;
    } else {
        debug!("Successfully copied static dir to dist");
    }
}
