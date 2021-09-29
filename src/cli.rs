use structopt::StructOpt;

use entropy::poacher;
use entropy::poacher::cli::PoachCmd;
use entropy::web;

#[derive(StructOpt, Debug)]
pub enum WebCmd {
    /// Run development server
    Dev,
    /// Build the public static site
    Build,
}

#[derive(StructOpt, Debug)]
pub enum CliCmd {
    /// Manage scrappers for aggregating content from web
    Poach(PoachCmd),
    /// Manage entropy web apps
    Web(WebCmd),
}

#[derive(StructOpt, Debug)]
#[structopt(name = "entropy", about = "Manage the entropy website")]
pub struct CliOpts {
    #[structopt(short, global = true, parse(from_occurrences))]
    pub verbosity: i32,

    #[structopt(subcommand)]
    pub cmd: CliCmd,
}

pub async fn run(cmd: CliCmd) -> anyhow::Result<()> {
    match cmd {
        CliCmd::Poach(poach_opts) => poacher::cli::run(poach_opts).await?,
        CliCmd::Web(web_cmd) => match web_cmd {
            WebCmd::Dev => web::run().await,
            WebCmd::Build => web::build().await?,
        },
    };

    Ok(())
}
