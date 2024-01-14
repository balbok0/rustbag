use std::{sync::Arc, path::Path, collections::HashMap};

use anyhow::{self, Result};
use object_store::{ObjectMeta, ObjectStore};
use tokio::sync::OnceCell;

use crate::{meta::Meta, records::{record::{Record, parse_header_bytes}, bag_header::BagHeader, connection::Connection}, cursor::Cursor, constants::{VERSION_LEN, VERSION_STRING}, error::RosError};
use url::Url;

#[derive()]
pub struct Bag {
    bag_meta: OnceCell<Meta>,
    bag_header: BagHeader,
    cursor: Cursor,
}


impl Bag {
    pub async fn try_new_from_object_store_meta(object_store: Arc<Box<dyn ObjectStore>>, object_meta: ObjectMeta) -> Result<Self> {
        let cursor = Cursor::new(object_store, object_meta);

        let bag_header = read_bag_header(&cursor).await?;
        Ok(Bag {
            bag_meta: OnceCell::new(),
            bag_header,
            cursor: cursor,
        })
    }

    pub async fn try_new_from_url(url: &Url) -> Result<Self> {
        let (obj_store, object_path) = object_store::parse_url(url)?;
        let object_meta = obj_store.head(&object_path).await?;

        Bag::try_new_from_object_store_meta(Arc::new(obj_store), object_meta).await
    }

    pub async fn try_from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let obj_store = object_store::local::LocalFileSystem::new();
        let obj_path = object_store::path::Path::from_filesystem_path(path)?;
        let obj_meta = obj_store.head(&obj_path).await?;

        Bag::try_new_from_object_store_meta(Arc::new(Box::new(obj_store)), obj_meta).await
    }

    pub async fn test_read(&self) -> Result<()> {
        let bag_header = read_bag_header(&self.cursor).await?;

        println!("Index pos: {}", bag_header.index_pos());

        Ok(())
    }

    pub async fn connections_by_topic(&self) -> Result<&HashMap<String, Vec<Connection>>> {
        let meta = self.borrow_meta().await;

        Ok(&meta.topic_to_connections)
    }

    pub async fn topics(&self) -> Vec<&String> {
        let meta = self.borrow_meta().await;

        let topics: Vec<_> = meta.topic_to_connections.keys().collect();

        topics
    }

    async fn borrow_meta(&self) -> &Meta {
        let meta = self.bag_meta.get_or_try_init(
            || async {
                let index_pos = self.bag_header.index_pos() as usize;
                Meta::try_new_from_bytes(self.cursor.read_bytes(index_pos, self.cursor.len() - index_pos).await?)
            }
        ).await;

        if meta.is_err() {
            panic!("Could not read Bag metadata {:#?}", meta)
        }

        meta.unwrap()
    }
}


// Helper Function
async fn read_bag_header(cursor: &Cursor) -> Result<BagHeader> {
    let bag_version_header = cursor.read_bytes(0, VERSION_LEN).await?;
    if bag_version_header != VERSION_STRING {
        return Err(RosError::InvalidVersion.into());
    }
    let header_bytes = cursor.read_chunk(VERSION_LEN).await?;
    let data_pos = 4 + header_bytes.len() + VERSION_LEN;
    let record = parse_header_bytes(data_pos, header_bytes)?;
    if let Record::BagHeader(bh) = record {
        Ok(bh)
    } else {
        Err(RosError::InvalidHeader("Invalid Bag Header record type.").into())
    }
}