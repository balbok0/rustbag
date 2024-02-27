use std::{sync::Arc, path::Path, collections::HashMap};

use anyhow::{self, Result};
use indicatif::ProgressBar;
use object_store::{ObjectMeta, ObjectStore};
use ros_msg::{msg_type::MsgType, msg_value::{FieldValue, MsgValue}, traits::ParseBytes as _};
use tokio::{runtime::Runtime, sync::OnceCell};

use crate::{meta::Meta, records::{record::{Record, parse_header_bytes, self}, bag_header::BagHeader, connection::Connection, chunk::ChunkData}, cursor::Cursor, constants::{VERSION_LEN, VERSION_STRING}, error::RosError};
use url::Url;

#[derive(Debug, Clone)]
pub struct Bag {
    bag_meta: OnceCell<Meta>,
    bag_header: OnceCell<BagHeader>,
    cursor: Cursor,
}


impl Bag {
    pub fn try_new_from_object_store_meta(object_store: Arc<Box<dyn ObjectStore>>, object_meta: ObjectMeta) -> Result<Self> {
        let cursor = Cursor::new(object_store, object_meta);

        Ok(Bag {
            bag_meta: OnceCell::new(),
            bag_header: OnceCell::new(),
            cursor,
        })
    }

    pub async fn try_new_from_url(url: &Url) -> Result<Self> {
        let (obj_store, object_path) = object_store::parse_url(url)?;
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
        self.bag_header.get_or_try_init(|| async {
            read_bag_header(&self.cursor).await
        }).await
    }

    async fn borrow_meta(&self) -> &Meta {
        let meta = self.bag_meta.get_or_try_init(
            || async {
                let index_pos = self.borrow_bag_header().await?._index_pos as usize;
                Meta::try_new_from_bytes(self.cursor.read_bytes(index_pos, self.cursor.len() - index_pos).await?)
            }
        ).await;

        if meta.is_err() {
            panic!("Could not read Bag metadata {:#?}", meta)
        }

        meta.unwrap()
    }

    pub async fn read_messages(&'_ self, topics: Option<Vec<String>>, start: Option<u64>, end: Option<u64>, verbose: bool) -> BagMessageIterator {
        let meta = self.borrow_meta().await;
        let start = start.map(|v| meta.start_time() + v * 1_000_000_000).unwrap_or_else(|| meta.start_time());
        let end = end.map(|v| meta.end_time() + v * 1_000_000_000).unwrap_or_else(|| meta.end_time());

        let chunk_positions = meta.filter_chunks(topics.as_ref(), Some(start), Some(end)).unwrap();

        let iter = BagMessageIterator::new(self.clone(), meta.clone(), start, end, chunk_positions, verbose);

        iter
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


// Helper struct for iteration of msgs
pub struct BagMessageIterator {
    inner: Bag,
    start: u64,
    end: u64,
    chunk_positions: Vec<u64>,
    con_to_msg: HashMap<u32, MsgType>,

    runtime: Runtime,

    chunk_index: usize,
    msg_index: usize,
    chunk_data: Option<ChunkData>,

    progress_bar: Option<ProgressBar>,
    last_timestamp: u64,
}

impl BagMessageIterator {
    fn new(bag: Bag, meta: Meta, start: u64, end: u64, chunk_positions: Vec<u64>, verbose: bool) -> Self {
        let con_to_msg = meta.borrow_connection_to_id_message();
        let progress_bar = if verbose {
            Some(indicatif::ProgressBar::new((end - start) / 1_000_000_000))
        } else {
            None
        };

        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();

        BagMessageIterator {
            inner: bag,
            start,
            end,
            chunk_positions,
            con_to_msg: con_to_msg.clone(),
            runtime,
            chunk_index: 0,
            msg_index: 0,
            chunk_data: None,
            progress_bar,
            last_timestamp: start,
        }
    }
}

impl Iterator for BagMessageIterator {
    type Item = MsgValue;

    fn next(&mut self) -> Option<Self::Item> {
        let bag = &self.inner;

        if self.chunk_data.as_ref().map(|cd| cd.message_datas.len() <= self.msg_index).unwrap_or(true) {
            println!("in if");
            if self.chunk_index >= self.chunk_positions.len() {
                return None;
            }
            self.msg_index = 0;

            self.chunk_data = self.runtime.block_on(async {
                println!("in async a chunk");
                let pos = self.chunk_positions[self.chunk_index];
                let pos = pos as usize;
                let header_bytes = bag.cursor.read_chunk(pos).await.unwrap();
                let header_len = header_bytes.len();
                let data_pos = pos + 4 + header_len;
                let record_with_header = parse_header_bytes(data_pos, header_bytes).ok()?;


                let chunk_data = if let record::Record::Chunk(c) = record_with_header {
                    println!("Decompressing");
                    let chunk_bytes = c.decompress(bag.cursor.read_chunk(data_pos).await.ok()?).ok()?;

                    ChunkData::try_from_bytes_with_time_check(chunk_bytes, self.start, self.end).ok()
                } else {
                    println!("record not a chunk");
                    return None;
                };

                chunk_data
            });

            self.chunk_index += 1;

        }

        let msg_data = self.chunk_data.as_ref().unwrap().message_datas.get(self.msg_index)?;
        self.msg_index += 1;

        if let Some(pbar) = &self.progress_bar {
            pbar.inc(msg_data._time - self.last_timestamp);
        }
        self.last_timestamp = msg_data._time;
        let msg_val = match self.con_to_msg.get(&msg_data._conn).unwrap().try_parse(&msg_data.data.clone().unwrap()) {
            Ok((_, FieldValue::Msg(msg))) => Some(msg),
            _ => None,
        };

        msg_val
    }
}