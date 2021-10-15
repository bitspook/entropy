use structopt::StructOpt;

use super::Local;

#[derive(StructOpt, Debug)]
pub enum Cmd {
    Groups,
    Events,
}

pub async fn run(cmd: Cmd, local: Local) -> anyhow::Result<()> {
    match cmd {
        Cmd::Groups => {
            local.poach_groups().await?;
        }
        Cmd::Events => {
            local.poach_events().await?;
        }
    }

    Ok(())
}
