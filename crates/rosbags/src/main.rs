mod config;

use clap::Parser;
use config::Args;

#[tokio::main]
async fn main() {
    println!("Hello, world!");

    let args = Args::parse();
}
