use structopt::StructOpt;

use crate::{db, poacher::meetup};

use super::Meetup;

#[derive(StructOpt, Debug)]
pub enum MeetupCmd {
    Groups,
    Events,
}

pub async fn run(cmd: MeetupCmd, meetup: Meetup) -> anyhow::Result<()> {
    let conn = db::establish_connection()?;

    meetup::clear_poached_data(&conn).await;
    info!("Cleared data poached from meetup.com");

    match cmd {
        MeetupCmd::Groups => {
            meetup.search_groups().await;
        }
        MeetupCmd::Events => {
            meetup.search_events().await;
        }
    }

    // Explicitly drop meetup so that tx can be dropped, signaling rx it can
    // stop. Otherwise consumer would run forever since rx would keep waiting
    // for new messages from tx
    drop(meetup);

    Ok(())
}
