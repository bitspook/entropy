use rocket_sync_db_pools::database;

mod build;
mod routes;
mod server;
mod utils;

mod config;

pub use build::*;
pub use config::*;
pub use server::*;

#[database("entropy_db")]
struct Db(diesel::PgConnection);

type WebResult<T> = Result<T, rocket::response::Debug<anyhow::Error>>;
