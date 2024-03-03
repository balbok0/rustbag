mod config;
use std::sync::Arc;

use anyhow::Result;

use clap::Parser;
use config::Args;
use indicatif;
use object_store;
use object_store::ObjectStore;
use rustbag;

fn main() -> Result<()> {
    let args = Args::parse();
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    let minio_url = "http://localhost:9000";
    let store = object_store::aws::AmazonS3Builder::new()
        .with_endpoint(minio_url)
        .with_region("minio")
        .with_bucket_name("test-bags")
        .with_access_key_id("minioadmin")
        .with_secret_access_key("minioadmin")
        .with_allow_http(true)
        .build()?;

    let bag = runtime.block_on(async {
        let obj_path = object_store::path::Path::from(args.bag_path);
        let object_meta = store.head(&obj_path).await.unwrap();

        rustbag::Bag::try_new_from_object_store_meta(
            Arc::new(Box::new(store)),
            object_meta,
        )
        .unwrap()
    });

    let msg_iter = runtime.block_on(async {
        bag.read_messages(None, args.start_ts, args.end_ts, rustbag::bag_msg_iterator::BagMessageIteratorConfig::default())
            .await
    });

    let num_msgs = runtime.block_on(async { bag.num_messages().await });

    let pbar = indicatif::ProgressBar::new(num_msgs);

    for _msg in msg_iter {
        pbar.inc(1);
    }

    Ok(())
}
