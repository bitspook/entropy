use anyhow::Error;
use diesel::pg::PgConnection;
use diesel::prelude::*;

pub mod models;
pub mod schema;

pub fn establish_connection() -> anyhow::Result<PgConnection> {
    let config = crate::EntropyConfig::load()?;
    let database_url = config.database_url;

    let conn = PgConnection::establish(&database_url).map_err(Error::from)?;

    Ok(conn)
}
