use anyhow::Result;
use structopt::StructOpt;
use tokio::{
    self,
    sync::mpsc::{self, Receiver, Sender},
};

use crate::EntropyConfig;

use super::{
    consumer,
    local::cli::Cmd as LocalPoacherCmd,
    local::{self, Local as LocalPoacher},
    meetup::{self, cli::MeetupCmd, Meetup},
    PoacherMessage,
};

#[derive(StructOpt, Debug)]
pub enum PoachCmd {
    Meetup(MeetupCmd),
    Local(LocalPoacherCmd),
}

pub async fn run(cmd: PoachCmd) -> Result<()> {
    let user_agent = "Mozilla/5.0 (X11; Linux x86_64; rv:88.0) Gecko/20100101 Firefox/88.0";
    let client = reqwest::Client::builder()
        .user_agent(user_agent)
        .build()
        .unwrap();
    let config = EntropyConfig::load().unwrap();

    let (tx, rx): (Sender<PoacherMessage>, Receiver<PoacherMessage>) = mpsc::channel(1024);

    let meetup = Meetup::new(client, config.poacher.meetup_com.to_vec(), tx.clone());
    let local_poacher = LocalPoacher::new(config.poacher.local, tx.clone());

    match cmd {
        PoachCmd::Meetup(opts) => meetup::cli::run(opts, meetup).await?,
        PoachCmd::Local(opts) => local::cli::run(opts, local_poacher).await?,
    };

    let groups_blacklist: Vec<String> = config
        .poacher
        .meetup_com
        .into_iter()
        .flat_map(|mc| mc.blacklist.groups.slugs)
        .collect();

    consumer::run(rx, &groups_blacklist).await?;

    Ok(())
}
