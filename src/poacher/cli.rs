use structopt::StructOpt;

use super::meetup::cli::MeetupCmd;

#[derive(StructOpt, Debug)]
pub enum PoachCmd {
    Meetup(MeetupCmd),
}
