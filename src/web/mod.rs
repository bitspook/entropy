mod build;
mod routes;
mod server;
use rocket_sync_db_pools::database;

pub use build::*;
pub use server::*;

#[database("entropy_db")]
struct Db(diesel::SqliteConnection);

type WebResult<T> = Result<T, rocket::response::Debug<anyhow::Error>>;
