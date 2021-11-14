use anyhow::Context;
use chrono::NaiveDateTime;
use diesel::data_types::PgInterval;
use serde::Deserialize;
use std::convert::TryFrom;

use crate::db::models::{Initiative, NewEvent, NewEventSection, NewGoal, NewGroup};

use super::utils::FMatterSection;

#[derive(Debug, Deserialize)]
pub struct LocalGroup {
    pub name: String,
    pub slug: String,
    pub description: String,
}

#[derive(Debug, Deserialize)]
pub struct LocalEventSection {
    #[serde(alias = "section")]
    pub name: String,
    pub title: String,
    pub logo: Option<String>,
    pub description: String,
    pub start_time: NaiveDateTime,
    pub end_time: NaiveDateTime,
}

#[derive(Debug, Deserialize)]
pub struct LocalEvent {
    pub title: String,
    pub slug: String,
    pub description: String,
    pub group_slug: String,
    pub start_time: NaiveDateTime,
    pub end_time: NaiveDateTime,
}

#[derive(Debug, Deserialize)]
pub struct LocalInitiative {
    pub title: String,
    pub slug: String,
    pub description: String,
}

#[derive(Debug, Deserialize)]
pub struct LocalGoal {
    pub title: String,
    pub slug: String,
    pub description: String,
    pub target: i32,
    pub interval_in_minutes: u32,
}

impl Into<NewGroup> for LocalGroup {
    fn into(self) -> NewGroup {
        NewGroup {
            name: self.name,
            slug: self.slug,
            description: Some(self.description),
            desc_format: "md".to_string(),
            source: Some(super::SOURCE.to_string()),
            source_link: None,
        }
    }
}

impl From<LocalEvent> for NewEvent {
    fn from(event: LocalEvent) -> NewEvent {
        NewEvent {
            title: event.title,
            slug: event.slug,
            description: Some(event.description),
            group_id: None,
            desc_format: "md".to_string(),
            photos: vec![],
            source: Some(super::SOURCE.to_string()),
            source_link: None,
            start_time: event.start_time,
            end_time: event.end_time,
        }
    }
}

impl From<LocalEventSection> for NewEventSection {
    fn from(sec: LocalEventSection) -> Self {
        Self {
            name: sec.name,
            title: sec.title,
            logo: sec.logo,
            description: Some(sec.description),
            desc_format: "md".to_string(),
            start_time: sec.start_time,
            end_time: sec.end_time,
            event_id: None,
        }
    }
}

impl From<LocalInitiative> for Initiative {
    fn from(li: LocalInitiative) -> Self {
        Self {
            title: li.title,
            slug: li.slug,
            source: Some(super::SOURCE.to_string()),
            description: Some(li.description),
            desc_format: "md".to_string(),
        }
    }
}

impl From<LocalGoal> for NewGoal {
    fn from(lg: LocalGoal) -> Self {
        Self {
            title: lg.title,
            slug: lg.slug,
            description: Some(lg.description),
            desc_format: "md".to_string(),
            initiative_slug: None,
            target: lg.target,
            iteration_interval: Some(PgInterval::from_microseconds(
                (lg.interval_in_minutes as i64) * 60 * 1000000,
            )),
        }
    }
}

impl TryFrom<FMatterSection> for LocalEvent {
    type Error = anyhow::Error;

    fn try_from(mut sec: FMatterSection) -> Result<Self, Self::Error> {
        let event = sec
            .meta
            .as_table_mut()
            .with_context(|| "Section meta should be a Map")?;

        // When front-matter is converted to `LocalEvent` it
        // fails because it don't have `description` field.
        // Content of the top-level section is used as
        // `description` here
        event.insert("description".to_string(), toml::Value::String(sec.content));
        event.insert("sections".to_string(), toml::Value::Array(vec![]));

        toml::Value::Table(event.clone())
            .try_into()
            .map_err(anyhow::Error::from)
            .with_context(|| "Failed to convert FMatterSection to LocalEvent")
    }
}

impl TryFrom<FMatterSection> for LocalEventSection {
    type Error = anyhow::Error;

    fn try_from(mut sec: FMatterSection) -> Result<Self, Self::Error> {
        let event_section = sec
            .meta
            .as_table_mut()
            .with_context(|| "Section meta should be a Map")?;

        event_section.insert("description".to_string(), toml::Value::String(sec.content));

        toml::Value::Table(event_section.clone())
            .try_into()
            .map_err(anyhow::Error::from)
            .with_context(|| "Failed to convert FMatterSection to LocalEventSection")
    }
}

impl TryFrom<FMatterSection> for LocalGroup {
    type Error = anyhow::Error;

    fn try_from(mut sec: FMatterSection) -> Result<Self, Self::Error> {
        let group = sec
            .meta
            .as_table_mut()
            .with_context(|| "Section meta should be a Map")?;

        group.insert("description".to_string(), toml::Value::String(sec.content));

        toml::Value::Table(group.clone())
            .try_into()
            .map_err(anyhow::Error::from)
            .with_context(|| "Failed to convert FMatterSection to LocalGroup")
    }
}

impl TryFrom<FMatterSection> for LocalInitiative {
    type Error = anyhow::Error;

    fn try_from(mut sec: FMatterSection) -> Result<Self, Self::Error> {
        let initiative = sec
            .meta
            .as_table_mut()
            .with_context(|| "Section meta should be a Map")?;

        initiative.insert("description".to_string(), toml::Value::String(sec.content));

        toml::Value::Table(initiative.clone())
            .try_into()
            .map_err(anyhow::Error::from)
            .with_context(|| "Failed to convert FMatterSection to LocalInitiative")
    }
}

impl TryFrom<FMatterSection> for LocalGoal {
    type Error = anyhow::Error;

    fn try_from(mut sec: FMatterSection) -> Result<Self, Self::Error> {
        let goal = sec
            .meta
            .as_table_mut()
            .with_context(|| "Section meta should be a Map")?;

        goal.insert("description".to_string(), toml::Value::String(sec.content));

        toml::Value::Table(goal.clone())
            .try_into()
            .map_err(anyhow::Error::from)
            .with_context(|| "Failed to convert FMatterSection to LocalGoal")
    }
}
