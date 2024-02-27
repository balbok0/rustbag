mod config;
use anyhow::Result;

use clap::Parser;
use config::Args;
use rosbags_lib;

#[tokio::main]
async fn main() -> Result<()> {
    println!("Hello, world!");

    let args = Args::parse();

    let bag = rosbags_lib::Bag::try_from_path(args.bag_path).await?;
    bag.connections_by_topic().await?;

    Ok(())
}
