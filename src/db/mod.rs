use anyhow::Error;
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;

pub mod schema;

pub fn establish_connection() -> anyhow::Result<SqliteConnection> {
    let config = crate::EntropyConfig::load()?;
    let database_path = config.database_path;

    let conn  = SqliteConnection::establish(&database_path).map_err(Error::from)?;

    Ok(conn)
}
