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
    let user_agent = "Mozilla/5.0 (X11; Linux x86_64; rv:88.0) Gecko/20100101 Firefox/88.0";
    let client = reqwest::Client::builder()
        .user_agent(user_agent)
        .build()
        .unwrap();
    let config = EntropyConfig::load().unwrap();

    let (tx, rx): (Sender<PoacherMessage>, Receiver<PoacherMessage>) = mpsc::channel(1024);
    let meetup = Meetup::new(client, config.poacher.meetup_com.to_vec(), tx);

    match cmd {
        PoachCmd::Meetup(poach_meetup_opts) => match poach_meetup_opts {
            MeetupCmd::Groups => {
                meetup.search_groups().await;
            }
            MeetupCmd::Events => {
                meetup.search_events().await;
            }
        },
    };

    let groups_blacklist: Vec<String> = config
        .poacher
        .meetup_com
        .into_iter()
        .flat_map(|mc| mc.blacklist.groups.slugs)
        .collect();

    // Explicitly drop meetup so that tx can be dropped, signaling rx it can
    // stop. Otherwise consumer would run forever since rx would keep waiting
    // for new messages from tx
    drop(meetup);
    consumer::run(rx, &groups_blacklist).await?;

    Ok(())
}
