use std::{collections::{BinaryHeap, HashMap, VecDeque}, path::Path, sync::Arc, task::Poll};

use anyhow::{self, Result};
use futures::FutureExt;
use object_store::{ObjectMeta, ObjectStore};
use ros_msg::{msg_type::MsgType, msg_value::{FieldValue, MsgValue}, traits::ParseBytes as _};
use tokio::{runtime::Runtime, sync::{mpsc::{Receiver, Sender}, oneshot::Receiver as OneShotReceiver, OnceCell}, task::JoinSet};

use crate::{meta::Meta, records::{bag_header::BagHeader, chunk::ChunkData, chunk_info::ChunkInfo, connection::Connection, record::{Record, parse_header_bytes, self}}, cursor::Cursor, constants::{VERSION_LEN, VERSION_STRING}, error::RosError};
use url::Url;

type MsgIterValue = (u64, u32, MsgValue);

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

    pub async fn read_messages(&self, topics: Option<Vec<String>>, start: Option<u64>, end: Option<u64>) -> BagMessageIterator {
        let meta = self.borrow_meta().await;
        let start = start.map(|v| meta.start_time() + v * 1_000_000_000).unwrap_or_else(|| meta.start_time());
        let end = end.map(|v| meta.end_time() + v * 1_000_000_000).unwrap_or_else(|| meta.end_time());

        let chunk_infos = meta.filter_chunks(topics.as_ref(), Some(start), Some(end)).unwrap();

        let iter = BagMessageIterator::new(self.clone(), meta.clone(), start, end, chunk_infos.into_iter().cloned().collect());

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


// Helper struct for iteration of msgs
pub struct BagMessageIterator {
    runtime: Runtime,
    message_reader: Receiver<Option<Vec<MsgIterValue>>>,
    msg_queue: VecDeque<MsgIterValue>,
}

pub(super) async fn start_parse_msgs(bag: Bag, chunk_infos: Vec<ChunkInfo>, con_to_msg: HashMap<u32, MsgType>, start: u64, end: u64, message_sender: Sender<Option<Vec<MsgIterValue>>>) {
    let (tx, chunk_result_recv) = tokio::sync::mpsc::channel(100);
    let (done_tx, done_recv) = tokio::sync::oneshot::channel();

    let sorted_fut = tokio::spawn(async move { order_parsed_messaged(chunk_result_recv, done_recv, message_sender).await.unwrap();  });

    // Chunk parsing
    let mut futures = JoinSet::new();

    for chunk_idx in 0..chunk_infos.len() {
        if futures.len() >= 10 {
            // Wait for some future to finish
            match futures.join_next().await {
                Some(v) => {
                    // FIXME: Do not fail silently
                    if v.is_err() {
                    }
                }
                None => {
                    // FIXME: Do not fail silently
                    return;
                }
            }
        }

        let chunk_info = &chunk_infos[chunk_idx];
        let chunk_pos = chunk_info._chunk_pos as usize;
        // TODO: Logic for waiting

        let chunk_con_to_msg = HashMap::from_iter(chunk_info.data.get().unwrap().iter().map(|c| (c._conn, con_to_msg.get(&c._conn).unwrap().clone())));

        let cur_tx = tx.clone();
        let chunk_bag = bag.clone();

        futures.spawn(async move {
            parse_chunk(
                cur_tx,
                chunk_bag,
                chunk_idx,
                chunk_pos,
                start,
                end,
                chunk_con_to_msg,
            ).await.unwrap();
        });
    }

    // Make sure all parsing is done
    while !futures.is_empty() {
        futures.join_next().await;
    }

    sorted_fut.await.unwrap();

    done_tx.send(true).unwrap();
}


async fn order_parsed_messaged(mut chunk_result_recv: Receiver<(usize, Vec<MsgIterValue>)>, mut done_recv: OneShotReceiver<bool>, sorted_result_sender: Sender<Option<Vec<MsgIterValue>>>) -> Result<()> {
    let waker = futures::task::noop_waker_ref();
    let mut cx = std::task::Context::from_waker(waker);
    let mut next_idx = 0;
    let mut last_printed_chunk_idx = 0;

    let mut parsed_ooo_chunks = BinaryHeap::new();


    loop {
        // Check if we are done
        match done_recv.poll_unpin(&mut cx) {
            Poll::Ready(Ok(true)) | Poll::Ready(Err(_)) => {
                sorted_result_sender.try_send(None).unwrap();
                break;
            },
            _ => {},
        }

        // If not check for any futures
        match chunk_result_recv.try_recv() {
            Ok((chunk_idx, msg_vals)) => {
                if chunk_idx == next_idx {
                    next_idx += 1;
                    sorted_result_sender.try_send(Some(msg_vals)).unwrap();
                } else {
                    parsed_ooo_chunks.push((-(chunk_idx as i64), msg_vals))
                }
            },
            Err(tokio::sync::mpsc::error::TryRecvError::Empty) => {},
            Err(e) => {
                // FIXME: Should break I think
                panic!("{:?}", e);
            }
        }

        // Lastly peek at OOO chunks to see if they should be added
        loop {
            match parsed_ooo_chunks.peek() {
                Some((chunk_idx, _)) => {
                    // DEBUG
                    if &last_printed_chunk_idx != chunk_idx {
                        last_printed_chunk_idx = *chunk_idx;
                    }

                    if -chunk_idx as usize == next_idx {
                        let (_, msg_vals) = parsed_ooo_chunks.pop().unwrap();
                        next_idx += 1;
                        sorted_result_sender.try_send(Some(msg_vals)).unwrap();
                    } else {
                        break;
                    }
                },
                None => break,
            }
        }
    }

    Ok(())

}

async fn parse_chunk(
    tx: Sender<(usize, Vec<MsgIterValue>)>,
    bag: Bag,
    chunk_idx: usize,
    pos: usize,
    start: u64,
    end: u64,
    con_to_msg: HashMap<u32, MsgType>,
) -> Result<()> {
    let header_bytes = bag.cursor.read_chunk(pos).await.unwrap();
    let header_len = header_bytes.len();
    let data_pos = pos + 4 + header_len;
    let record_with_header = parse_header_bytes(data_pos, header_bytes)?;

    let chunk_data = if let record::Record::Chunk(c) = record_with_header {
        let chunk_bytes = c.decompress(bag.cursor.read_chunk(data_pos).await.unwrap()).unwrap();

        ChunkData::try_from_bytes_with_time_check(chunk_bytes, start, end).unwrap()
    } else {
        return Err(anyhow::Error::new(RosError::InvalidRecord("Bad Record type detected. Expected Chunk.")));
    };

    let mut message_vals = Vec::with_capacity(chunk_data.message_datas.len());
    for md in chunk_data.message_datas {
        let msg_val = match con_to_msg.get(&md._conn).unwrap().try_parse(&md.data.unwrap()) {
            Ok((_, FieldValue::Msg(msg))) => msg,
            _ => {
                return Err(anyhow::Error::new(RosError::InvalidRecord("MessageData did not contain a parsable Value")));
            }
        };
        message_vals.push((md._time, md._conn, msg_val));
    }


    tx.send((chunk_idx, message_vals)).await.unwrap();

    Ok(())
}

impl BagMessageIterator {
    fn new(bag: Bag, meta: Meta, start: u64, end: u64, chunk_infos: Vec<ChunkInfo>) -> Self {
        let con_to_msg = meta.borrow_connection_to_id_message();

        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .worker_threads(8)
            .build()
            .unwrap();

        let (message_sender, message_reader) = tokio::sync::mpsc::channel(1000);
        runtime.spawn(start_parse_msgs(bag, chunk_infos, con_to_msg.clone(), start, end, message_sender));

        let s = BagMessageIterator {
            runtime,
            message_reader,
            msg_queue: VecDeque::new(),
        };

        s
    }

}

impl Iterator for BagMessageIterator {
    type Item = MsgIterValue;

    fn next(&mut self) -> Option<Self::Item> {
        match self.msg_queue.pop_front() {
            Some(msg) => Some(msg),
            None => {
                match self.message_reader.blocking_recv() {
                    Some(Some(msgs)) => {
                        self.msg_queue.append(&mut msgs.into());
                        Some(self.msg_queue.pop_front().unwrap())
                    }
                    Some(None) | None => None,
                }
            }
        }
    }
}