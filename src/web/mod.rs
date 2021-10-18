mod build;
mod routes;
mod server;
mod utils;
use rocket_sync_db_pools::database;

pub use build::*;
pub use server::*;

#[database("entropy_db")]
struct Db(diesel::PgConnection);

type WebResult<T> = Result<T, rocket::response::Debug<anyhow::Error>>;
