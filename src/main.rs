use anyhow::Result;
use env_logger::Env;
use structopt::StructOpt;
use tokio;

use entropy::db;

#[macro_use]
extern crate diesel_migrations;

mod cli;

fn mk_logger(verbosity: i32) {
    let level = match verbosity {
        0 => "info",
        1 => "debug",
        _ => "trace",
    };
    let env = Env::default()
        .filter_or("ENTROPY_LOG_LEVEL", level)
        .write_style_or("ENTROPY_LOG_STYLE", "always");

    env_logger::init_from_env(env);
}

embed_migrations!();

#[tokio::main]
async fn main() -> Result<()> {
    let cli_opts = cli::CliOpts::from_args();

    mk_logger(cli_opts.verbosity);

    let conn = db::establish_connection()?;
    embedded_migrations::run(&conn)?;

    cli::run(cli_opts.cmd).await?;

    Ok(())
}
