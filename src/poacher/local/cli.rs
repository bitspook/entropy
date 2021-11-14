use structopt::StructOpt;

use crate::{db, poacher::local::SOURCE};

use super::Local;

#[derive(StructOpt, Debug)]
pub enum Cmd {
    All
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

pub async fn clear_poached_initiatives(conn: &diesel::PgConnection) {
    use crate::db::schema::*;
    use diesel::{delete, prelude::*};

    if let Err(err) =
        delete(initiatives::dsl::initiatives.filter(initiatives::dsl::source.eq(SOURCE))).execute(conn)
    {
        error!(
            "Error while clearing initiatives poached from {} [err={}]",
            SOURCE, err
        );
    } else {
        debug!("Cleared initiatives poached from {}", SOURCE);
    }
}

pub async fn clear_poached_data(conn: &diesel::PgConnection) {
    clear_poached_events(conn).await;
    clear_poached_groups(conn).await;
    clear_poached_initiatives(conn).await;
}

pub async fn run(cmd: Cmd, local: Local) -> anyhow::Result<()> {
    let conn = db::establish_connection()?;

    match cmd {
        Cmd::All => {
            clear_poached_data(&conn).await;
            local.poach_groups().await?;
            local.poach_events().await?;
            local.poach_initiatives().await?;
            local.end().await?;
        }
    }

    Ok(())
}
