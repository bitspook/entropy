use std::{fs::DirBuilder, io};

pub async fn build() {
    info!("Building public static website");

    let path = "dist";

    debug!("Creating dist directory");
    if let Err(err) = DirBuilder::new().create(path) {
        match err.kind() {
            io::ErrorKind::AlreadyExists => {
                debug!("dist already exists. Ignoring.")
            }
            _ => {
                error!("Failed to create dist dir: {:#}", err);
                return;
            }
        }
    } else {
        debug!("Successfully created dist dir");
    };
}
