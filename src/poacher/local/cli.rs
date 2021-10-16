use structopt::StructOpt;

use crate::{db, poacher::local::SOURCE};

use super::Local;

#[derive(StructOpt, Debug)]
pub enum Cmd {
    Groups,
    Events,
}

pub async fn clear_poached_events(conn: &diesel::PgConnection) {
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
}

pub async fn clear_poached_groups(conn: &diesel::PgConnection) {
    use crate::db::schema::*;
    use diesel::{delete, prelude::*};

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

// FIXME I have a feeling this is really hacky
pub async fn clear_poached_data(conn: &diesel::PgConnection) {
    clear_poached_events(conn).await;
    clear_poached_groups(conn).await;
}

pub async fn run(cmd: Cmd, local: Local) -> anyhow::Result<()> {
    let conn = db::establish_connection()?;

    match cmd {
        Cmd::Groups => {
            warn!("Poaching local groups will remove all already-poached local events. You might wanna poach local events next.");
            clear_poached_data(&conn).await;
            local.poach_groups().await?;
        }
        Cmd::Events => {
            clear_poached_events(&conn).await;
            local.poach_events().await?;
        }
    }

    Ok(())
}
