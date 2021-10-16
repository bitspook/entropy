use structopt::StructOpt;

use super::Meetup;
use crate::{db, poacher::meetup::SOURCE};

#[derive(StructOpt, Debug)]
pub enum MeetupCmd {
    Groups,
    Events,
}

async fn clear_poached_data(conn: &diesel::PgConnection) {
    use crate::db::schema::*;
    use diesel::{delete, prelude::*};

    if let Err(err) =
        delete(events::dsl::events.filter(events::dsl::source.eq(SOURCE))).execute(conn)
    {
        error!(
            "Error while clearing events poached from {} [err={}]",
            SOURCE, err
        );
    } else {
        debug!("Cleared events poached from {}", SOURCE);
    }

    if let Err(err) =
        delete(groups::dsl::groups.filter(groups::dsl::source.eq(SOURCE))).execute(conn)
    {
        error!(
            "Error while clearing groups poached from {} [err={}]",
            SOURCE, err
        );
    } else {
        debug!("Cleared groups poached from {}", SOURCE);
    }
}

pub async fn run(cmd: MeetupCmd, meetup: Meetup) -> anyhow::Result<()> {
    let conn = db::establish_connection()?;

    clear_poached_data(&conn).await;
    info!("Cleared data poached from meetup.com");

    match cmd {
        MeetupCmd::Groups => {
            meetup.search_groups().await;
        }
        MeetupCmd::Events => {
            meetup.search_events().await;
        }
    }

    Ok(())
}
