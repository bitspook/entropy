use anyhow::bail;
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
        _ => {
            bail!("Not implemented");
        }
    }

    Ok(())
}
