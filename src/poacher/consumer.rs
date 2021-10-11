use anyhow::bail;
use log::{debug, error};
use tokio::sync::mpsc::Receiver;

use crate::{db, poacher::meetup};

use super::{PoacherMessage, PoacherResult};

/// Absorb all the poacher messages from `rx` and spawn tasks to process them.
pub async fn run(
    mut rx: Receiver<PoacherMessage>,
    groups_blacklist: &Vec<String>,
) -> anyhow::Result<()> {
    let conn = db::establish_connection()?;
    while let Some(msg) = rx.recv().await {
        match msg {
            PoacherMessage::Error(err) => {
                error!("Encountered error when poaching: {:#?}", err)
            }
            PoacherMessage::Warning(w) => {
                warn!("Encountered warning: {:#?}", w)
            }
            PoacherMessage::End => {
                debug!("Received End poacher signal");
                // Break out of listen loop so receiver can go out of scope and be dropped.
                // If this don't happen, we'd keep waiting for next message and this loop
                // will never end.
                break;
            }
            PoacherMessage::ResultItem(item) => match item {
                PoacherResult::Meetup(result) => {
                    meetup::consumer::consume(result, &conn, groups_blacklist).await;
                }
                _ => {
                    bail!("Not Implemented");
                }
            },
        }
    }

    Ok(())
}
