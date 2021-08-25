use anyhow::Result;
use std::sync::Arc;
use structopt::StructOpt;
use tokio::{
    self,
    sync::mpsc::{self, Receiver, Sender},
};

use entropy::web;
use entropy::{Meetup, MeetupResult, PoachedResult, PoacherMessage};

use crate::db;
use crate::util::{
    process_scraped_meetup_event, process_scraped_meetup_group, search_events, search_groups,
};

#[derive(StructOpt, Debug)]
pub enum MeetupCmd {
    Groups,
    Events,
}

#[derive(StructOpt, Debug)]
pub enum PoachCmd {
    Meetup(MeetupCmd),
}

#[derive(StructOpt, Debug)]
pub enum WebCmd {
    /// Run development server
    Dev,
    /// Build the public static site
    Build,
}

#[derive(StructOpt, Debug)]
pub enum CliCmd {
    /// Manage scrappers for aggregating content from web
    Poach(PoachCmd),
    /// Manage entropy web apps
    Web(WebCmd),
}

#[derive(StructOpt, Debug)]
#[structopt(name = "entropy", about = "Manage the entropy website")]
pub struct CliOpts {
    #[structopt(short, global = true, parse(from_occurrences))]
    pub verbosity: i32,

    #[structopt(subcommand)]
    pub cmd: CliCmd,
}

pub async fn run(cmd: CliCmd) -> anyhow::Result<()> {
    match cmd {
        CliCmd::Poach(poach_opts) => {
            let (tx, rx): (Sender<PoacherMessage>, Receiver<PoacherMessage>) = mpsc::channel(1024);
            let user_agent = "Mozilla/5.0 (X11; Linux x86_64; rv:88.0) Gecko/20100101 Firefox/88.0";
            let client = reqwest::Client::builder()
                .user_agent(user_agent)
                .build()
                .unwrap();
            let meetup = Arc::new(Meetup::new(client, tx.clone()));

            match poach_opts {
                PoachCmd::Meetup(poach_meetup_opts) => match poach_meetup_opts {
                    MeetupCmd::Groups => {
                        search_groups(meetup, tx).await;
                    }
                    MeetupCmd::Events => {
                        search_events(meetup, tx).await;
                    }
                },
            };

            poacher_meditation(rx).await?;
        }
        CliCmd::Web(web_cmd) => match web_cmd {
            WebCmd::Dev => web::run().await,
            WebCmd::Build => web::build().await?,
        },
    };

    Ok(())
}

/// Absorb all the poacher messages from `rx` and spawn tasks to process them.
async fn poacher_meditation(mut rx: Receiver<PoacherMessage>) -> Result<()> {
    let conn = db::establish_connection()?;
    while let Some(msg) = rx.recv().await {
        match msg {
            PoacherMessage::Error(err) => {
                error!("Encountered error when poaching: {:#?}", err)
            }
            PoacherMessage::ResultItem(item) => match item {
                PoachedResult::Meetup(result) => match result {
                    MeetupResult::Group(group) => {
                        process_scraped_meetup_group(group, &conn).await;
                    }
                    MeetupResult::Event(event) => {
                        process_scraped_meetup_event(event, &conn).await;
                    }
                },
            },
            PoacherMessage::Warning(w) => {
                warn!("Encountered warning: {:#?}", w)
            }
        }
    }

    Ok(())
}
