use std::{sync::Arc, path::Path, collections::HashMap};

use anyhow::{self, Result};
use object_store::{ObjectMeta, ObjectStore};
use ros_msg::{msg_value::FieldValue, traits::ParseBytes as _};
use tokio::sync::OnceCell;

use crate::{meta::Meta, records::{record::{Record, parse_header_bytes, self}, bag_header::BagHeader, connection::Connection, chunk::ChunkData}, cursor::Cursor, constants::{VERSION_LEN, VERSION_STRING}, error::RosError};
use url::Url;

#[derive()]
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

    pub async fn read_messages(&self, topics: Option<Vec<String>>, start: Option<u64>, end: Option<u64>) -> Result<()> {
        let meta = self.borrow_meta().await;
        let start = start.map(|v| meta.start_time() + v * 1_000_000_000).unwrap_or_else(|| meta.start_time());
        let end = end.map(|v| meta.end_time() + v * 1_000_000_000).unwrap_or_else(|| meta.end_time());

        let chunk_positions = meta.filter_chunks(None, Some(start), Some(end))?;


        let bar = indicatif::ProgressBar::new(chunk_positions.len() as u64);
        let con_to_msg = meta.borrow_connection_to_id_message();
        for pos in chunk_positions {
            bar.inc(1);
            let pos = pos as usize;
            let header_bytes = self.cursor.read_chunk(pos).await.unwrap();
            let header_len = header_bytes.len();
            let data_pos = pos + 4 + header_len;
            let record_with_header = parse_header_bytes(data_pos, header_bytes)?;


            if let record::Record::Chunk(c) = record_with_header {
                let chunk_bytes = c.decompress(self.cursor.read_chunk(data_pos).await?)?;

                let chunk_data = ChunkData::try_from_bytes_with_time_check(chunk_bytes, start, end)?;

                for message_data in chunk_data.message_datas {
                    let msg = con_to_msg.get(&message_data._conn).unwrap().try_parse(&message_data.data.unwrap());
                    // println!("Message Data conn: {} Data len: {:?}", message_data._conn, &message_data.data.map(|d| d.len()));
                    // WARN: Slow!
                    // let msg = msg_map.get(&message_data._conn).unwrap().decode(message_data.data.unwrap().reader())?;
                }

            } else {
                return Err(RosError::InvalidRecord("Unexpected record. Expected Chunk").into());
            }
        }
        bar.finish();

        Ok(())
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
pub struct BagMessageIterator<'b> {
    inner: &'b Bag,
}

impl<'b> Iterator for BagMessageIterator<'b> {
    type Item = FieldValue;

    fn next(&mut self) -> Option<Self::Item> {
        // self.inner.
        todo!()
    }
}