use clap::Parser;

/// TODO: Write docs
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub(crate) struct Args {
    /// Path to bag
    pub(crate) bag_path: String,

    #[arg(short)]
    /// Timestamp to start reading messages from. Relative to start of the bag in seconds
    pub(crate) start_ts: Option<u64>,

    #[arg(short)]
    /// Timestamp to end reading messages at. Relative to start of the bag in seconds
    pub(crate) end_ts: Option<u64>,
}