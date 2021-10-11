//! Poach content from wherever entropy is ran. I intend to use this to provide
//! classic SSG behavior of reading markdown files and rendering them as web
//! pages.
use std::path::Path;

use anyhow::Context;
use tokio::{fs::read_dir, sync::mpsc::Sender};

use super::PoacherMessage;
pub mod cli;
mod config;

pub use config::*;

pub const SOURCE: &str = "local";

#[derive(Debug)]
pub struct LocalGroup {
    slug: String,
    name: String,
    description: String,
}

#[derive(Debug)]
pub struct LocalEvent {
    slug: String,
    title: String,
    description: String,
}

#[derive(Debug)]
pub enum LocalResult {
    Group(LocalGroup),
    Event(LocalEvent),
}

pub struct Local {
    config: Config,
    tx: Sender<PoacherMessage>,
}

impl Local {
    pub fn new(config: Config, tx: Sender<PoacherMessage>) -> Self {
        Self { config, tx }
    }

    pub async fn poach_groups(&self) -> Result<(), anyhow::Error> {
        let events_path = Path::new("./events");
        let mut files = read_dir(events_path)
            .await
            .with_context(|| format!("Failed to read [path={}]", events_path.to_string_lossy()))?;

        loop {
            let file = files.next_entry().await?;
            if let Some(entry) = file {
                info!("I got {:#?}", entry);
            } else {
                break;
            }
        }

        self.tx.send(PoacherMessage::End).await?;
        debug!("Done reading all groups. Sent end message");

        Ok(())
    }
}
