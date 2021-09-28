use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub enum MeetupCmd {
    Groups,
    Events,
}
