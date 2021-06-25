use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub enum MeetupCmd {
    Groups {
        #[structopt(long = "with-events")]
        with_events: bool,
        #[structopt(long, conflicts_with_all(&["lng", "lat"]))]
        /// One of the supported cities. lat, lng, radius and query for supported
        /// cities are already known to entropy and don't need to be provided
        city: Option<String>,
        #[structopt(long, required_unless("city"), requires("lng"))]
        /// Latitude of place around which groups/events should be found
        lat: Option<f32>,
        #[structopt(long, required_unless("city"), requires("lat"))]
        /// Longitude of place around which groups/events should be found
        lng: Option<f32>,
        #[structopt(long, conflicts_with("city"))]
        /// Radius in miles
        radius: Option<u32>,
        #[structopt(short, long, conflicts_with("city"))]
        /// Space separated list of queries you want to search the groups by
        query: Option<Vec<String>>,
    },
    Events {
        #[structopt(long = "group-slug")]
        group_slug: String,
        #[structopt(long = "group-id")]
        // Group-id requirement is silly and only exists because I needed
        // group-id to maintain a foreign key against every meetup-event. We can
        // get rid of this by making group-slug the key for every group, and use
        // that as a FK. I am leaving this here for now since this command will
        // probably not be used much, and I need to keep my momentum. Creating a
        // task (#7) to make group-slug the FK instead of group id and make this
        // option simpler.
        group_id: String,
    },
}

#[derive(StructOpt, Debug)]
pub enum PoachCmd {
    Meetup(MeetupCmd),
}

#[derive(StructOpt, Debug)]
pub enum CliCmd {
    Poach(PoachCmd),
}

#[derive(StructOpt, Debug)]
#[structopt(name = "entropy", about = "Manage the entropy website")]
pub struct CliOpts {
    #[structopt(short, global = true, parse(from_occurrences))]
    pub verbosity: i32,

    #[structopt(subcommand)]
    pub cmd: CliCmd,
}
