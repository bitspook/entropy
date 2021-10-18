//! Poach content from wherever entropy is ran. I intend to use this to provide
//! classic SSG behavior of reading markdown files and rendering them as web
//! pages.
use anyhow::Context;
use futures::pin_mut;
use regex::Regex;
use std::path::Path;
use tokio::sync::mpsc::Sender;
use tokio_stream::StreamExt;

use super::PoacherMessage;
use crate::poacher::{PoacherError, PoacherResult};
pub mod cli;
mod config;
mod models;
mod utils;
pub use config::*;
use models::*;
use utils::*;
pub mod consumer;

pub const SOURCE: &str = "local";

#[derive(Debug)]
pub enum LocalResult {
    Group(LocalGroup),
    Event(LocalEvent),
}

pub struct Local {
    config: LocalPoacherConfig,
    tx: Sender<PoacherMessage>,
}

impl Local {
    pub fn new(config: LocalPoacherConfig, tx: Sender<PoacherMessage>) -> Self {
        Self { config, tx }
    }

    pub async fn poach_events(&self) -> Result<(), anyhow::Error> {
        let base_dir = Path::new(&self.config.events.base_dir);
        // FIXME Shouldn't need to clone here
        let include = self
            .config
            .events
            .include
            .clone()
            .map(|s| Regex::new(&s).ok())
            .flatten();
        let exclude = self
            .config
            .events
            .exclude
            .clone()
            .map(|s| Regex::new(&s).ok())
            .flatten();

        let events = read_all_files(base_dir, include, exclude).await?;

        pin_mut!(events);

        while let Some(event) = events.next().await {
            match event {
                Ok(event) => {
                    let mut sections = into_toml_fmatter_sections(&event)
                        .with_context(|| "Error while parsing LocalEvent")?
                        .into_iter();

                    if let Some(mut top_section) = sections.next() {
                        let fmatter = top_section
                            .meta
                            .as_table_mut()
                            .with_context(|| "Failed to parse LocalEvent meta")?;

                        fmatter.insert(
                            "description".to_string(),
                            toml::Value::String(top_section.content),
                        );

                        let event: LocalEvent = toml::Value::Table(fmatter.clone())
                            .try_into()
                            .with_context(|| "Failed to parse LocalEvent")?;

                        self.tx
                            .send(PoacherMessage::ResultItem(PoacherResult::Local(
                                LocalResult::Event(event),
                            )))
                            .await?;
                    }
                }
                Err(err) => {
                    self.tx
                        .send(PoacherMessage::Error(PoacherError::UnknownError(
                            err.into(),
                        )))
                        .await?;
                }
            }
        }

        self.tx.send(PoacherMessage::End).await?;

        Ok(())
    }

    pub async fn poach_groups(&self) -> Result<(), anyhow::Error> {
        let base_dir = Path::new(&self.config.groups.base_dir);
        // FIXME Shouldn't need to clone here
        let include = self
            .config
            .groups
            .include
            .clone()
            .map(|s| Regex::new(&s).ok())
            .flatten();
        let exclude = self
            .config
            .groups
            .exclude
            .clone()
            .map(|s| Regex::new(&s).ok())
            .flatten();

        let groups = read_all_files(base_dir, include, exclude).await?;

        pin_mut!(groups);

        while let Some(group) = groups.next().await {
            match group {
                Ok(group) => {
                    let mut sections = into_toml_fmatter_sections(&group)
                        .with_context(|| "Error while extracting group sections")?
                        .into_iter();

                    if let Some(top_section) = sections.next() {
                        let mut group: LocalGroup = top_section.meta.try_into()?;
                        group.description = top_section.content;

                        self.tx
                            .send(PoacherMessage::ResultItem(PoacherResult::Local(
                                LocalResult::Group(group),
                            )))
                            .await?;
                    }
                }
                Err(err) => {
                    self.tx
                        .send(PoacherMessage::Error(PoacherError::UnknownError(
                            err.into(),
                        )))
                        .await?;
                }
            }
        }

        self.tx.send(PoacherMessage::End).await?;

        Ok(())
    }
}
