mod config;
use anyhow::Result;

use clap::Parser;
use config::Args;
use rosbags_lib;

fn main() -> Result<()> {
    let args = Args::parse();
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();


    let msg_iter = runtime.block_on(async {
        let bag = rosbags_lib::Bag::try_from_path(args.bag_path).await.unwrap();
        bag.read_messages(None, args.start_ts, args.end_ts, true).await
    });

    for msg in msg_iter {

    }

    Ok(())
}
