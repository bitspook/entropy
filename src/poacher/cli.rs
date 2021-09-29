use std::sync::Arc;

use anyhow::Result;
use structopt::StructOpt;
use tokio::{
    self,
    sync::mpsc::{self, Receiver, Sender},
};

use crate::EntropyConfig;

use super::{
    consumer,
    meetup::{cli::MeetupCmd, Meetup},
    PoacherMessage,
};

#[derive(StructOpt, Debug)]
pub enum PoachCmd {
    Meetup(MeetupCmd),
}

pub async fn run(cmd: PoachCmd) -> Result<()> {
    let (tx, rx): (Sender<PoacherMessage>, Receiver<PoacherMessage>) = mpsc::channel(1024);
    let user_agent = "Mozilla/5.0 (X11; Linux x86_64; rv:88.0) Gecko/20100101 Firefox/88.0";
    let client = reqwest::Client::builder()
        .user_agent(user_agent)
        .build()
        .unwrap();
    let config = EntropyConfig::load().unwrap();

    let meetup = Arc::new(Meetup::new(client, config.poacher.meetup_com, tx.clone()));
    let meetup2 = meetup.clone();

    match cmd {
        PoachCmd::Meetup(poach_meetup_opts) => match poach_meetup_opts {
            MeetupCmd::Groups => {
                meetup.search_groups(meetup2, tx).await;
            }
            MeetupCmd::Events => {
                meetup.search_events(meetup2, tx).await;
            }
        },
    };

    consumer::run(rx).await
}
