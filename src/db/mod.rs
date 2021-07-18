use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use std::env;
use std::env::VarError;

pub mod schema;

pub fn establish_connection() -> SqliteConnection {
    let default_db_url: Result<String, VarError> = Ok("entropy.sqlite3".to_owned());
    let database_url: String = env::var("DATABASE_URL").or(default_db_url).unwrap();

    SqliteConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
}
