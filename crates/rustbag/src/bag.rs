use std::{
    collections::HashMap,
    path::Path,
    sync::Arc,
};

use anyhow::{self, Result};
use object_store::{ObjectMeta, ObjectStore};
use tokio::sync::OnceCell;

use crate::{
    bag_msg_iterator::BagMessageIteratorConfig, constants::{VERSION_LEN, VERSION_STRING}, cursor::Cursor, error::RosError, meta::Meta, records::{
        bag_header::BagHeader,
        connection::Connection,
        record::{parse_header_bytes, Record},
    }, BagMessageIterator
};
use url::Url;

#[derive(Debug, Clone)]
pub struct Bag {
    bag_meta: OnceCell<Meta>,
    bag_header: OnceCell<BagHeader>,
    pub(crate) cursor: Cursor,
}

impl Bag {
    pub fn try_new_from_object_store_meta(
        object_store: Arc<Box<dyn ObjectStore>>,
        object_meta: ObjectMeta,
    ) -> Result<Self> {
        let cursor = Cursor::new(object_store, object_meta);

        Ok(Bag {
            bag_meta: OnceCell::new(),
            bag_header: OnceCell::new(),
            cursor,
        })
    }

    pub async fn try_new_from_url<I, K, V>(url: &Url, options: Option<I>) -> Result<Self>
    where
        I: IntoIterator<Item = (K, V)>,
        K: AsRef<str>,
        V: Into<String>,
    {
        let (obj_store, object_path) = options.map(|opts| {
            object_store::parse_url_opts(url, opts)
        }).unwrap_or_else(|| {
            object_store::parse_url(url)
        })?;
        let object_meta = obj_store.head(&object_path).await?;

        Bag::try_new_from_object_store_meta(Arc::new(obj_store), object_meta)
    }


    pub async fn try_from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let obj_store = object_store::local::LocalFileSystem::new();
        let obj_path = object_store::path::Path::from_filesystem_path(path)?;
        let obj_meta = obj_store.head(&obj_path).await?;

        Bag::try_new_from_object_store_meta(Arc::new(Box::new(obj_store)), obj_meta)
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

    async fn borrow_bag_header(&self) -> Result<&BagHeader> {
        self.bag_header
            .get_or_try_init(|| async { read_bag_header(&self.cursor).await })
            .await
    }

    async fn borrow_meta(&self) -> &Meta {
        let meta = self
            .bag_meta
            .get_or_try_init(|| async {
                let index_pos = self.borrow_bag_header().await?._index_pos as usize;
                Meta::try_new_from_bytes(
                    self.cursor
                        .read_bytes(index_pos, self.cursor.len() - index_pos)
                        .await?,
                )
            })
            .await;

        if meta.is_err() {
            panic!("Could not read Bag metadata {:#?}", meta)
        }

        meta.unwrap()
    }

    pub async fn read_messages(
        &self,
        topics: Option<Vec<String>>,
        start: Option<u64>,
        end: Option<u64>,
        config: BagMessageIteratorConfig,
    ) -> BagMessageIterator {
        let meta = self.borrow_meta().await;
        let start = start
            .map(|v| meta.start_time() + v * 1_000_000_000)
            .unwrap_or_else(|| meta.start_time());
        let end = end
            .map(|v| meta.end_time() + v * 1_000_000_000)
            .unwrap_or_else(|| meta.end_time());

        let chunk_infos = meta
            .filter_chunks(topics.as_ref(), Some(start), Some(end))
            .unwrap();

        let iter = BagMessageIterator::new(
            self.clone(),
            meta.clone(),
            start,
            end,
            chunk_infos.into_iter().cloned().collect(),
            config,
        );

        iter
    }

    pub async fn num_messages(&self) -> u64 {
        self.borrow_meta().await.num_messages()
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
