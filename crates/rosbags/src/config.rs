use clap::Parser;

/// TODO: Write docs
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub(crate) struct Args {
    /// Path to bag
    bag_path: String,
}