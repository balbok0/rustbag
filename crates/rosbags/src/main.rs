mod config;
use anyhow::Result;

use clap::Parser;
use config::Args;
use indicatif;
use rosbags_lib;

fn main() -> Result<()> {
    let args = Args::parse();
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    let bag = runtime.block_on(async {
        rosbags_lib::Bag::try_from_path(args.bag_path).await.unwrap()
    });

    let msg_iter = runtime.block_on(async {
        bag.read_messages(None, args.start_ts, args.end_ts).await
    });

    let num_msgs = runtime.block_on(async { bag.num_messages().await });

    let pbar = indicatif::ProgressBar::new(num_msgs);

    for msg in msg_iter {
        pbar.inc(1);

    }

    Ok(())
}
