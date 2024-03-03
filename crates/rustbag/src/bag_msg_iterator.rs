use std::collections::{BinaryHeap, HashMap, VecDeque};

use anyhow::{self, Result};
use ros_msg::{
    msg_type::MsgType,
    msg_value::FieldValue,
    traits::ParseBytes as _,
};
use tokio::{
    runtime::Runtime,
    sync::mpsc::{Receiver, Sender},
    task::JoinSet,
};

use crate::{
    constants::MsgIterValue, error::RosError, meta::Meta, records::{
        chunk::ChunkData,
        chunk_info::ChunkInfo,
        record::{self, parse_header_bytes},
    }, Bag
};

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct BagMessageIteratorConfig {
    pub num_threads: u32,
}

impl Default for BagMessageIteratorConfig {
    fn default() -> Self {
        Self { num_threads: 4 }
    }
}

impl From<HashMap<String, String>> for BagMessageIteratorConfig {
    fn from(value: HashMap<String, String>) -> Self {
        BagMessageIteratorConfig {
            num_threads: value.get("num_threads").map(|v| v.parse().unwrap()).unwrap_or(4)
        }
    }
}


#[derive(Debug)]
pub struct BagMessageIterator {
    _runtime: Runtime,
    message_reader: Receiver<Option<Vec<MsgIterValue>>>,
    msg_queue: VecDeque<MsgIterValue>,
    config: BagMessageIteratorConfig,
}

pub(super) async fn start_parse_msgs(
    bag: Bag,
    chunk_infos: Vec<ChunkInfo>,
    con_to_msg: HashMap<u32, MsgType>,
    start: u64,
    end: u64,
    message_sender: Sender<Option<Vec<MsgIterValue>>>,
) {
    let (tx, chunk_result_recv) = tokio::sync::mpsc::channel(10);

    let sorted_fut = tokio::spawn(async move {
        order_parsed_messaged(chunk_result_recv, message_sender)
            .await
            .unwrap();
    });

    // Chunk parsing
    let mut futures = JoinSet::new();

    for chunk_idx in 0..chunk_infos.len() {
        if futures.len() >= 100 {
            // Wait for some future to finish
            match futures.join_next().await {
                Some(v) => {
                    // FIXME: Do not fail silently
                    if v.is_err() {}
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

        let chunk_con_to_msg = HashMap::from_iter(
            chunk_info
                .data
                .get()
                .unwrap()
                .iter()
                .map(|c| (c._conn, con_to_msg.get(&c._conn).unwrap().clone())),
        );

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
            )
            .await
            .unwrap();
        });
    }
    // println!("Num chunks parsed: {}", chunk_infos.len());

    // Make sure all parsing is done
    while !futures.is_empty() {
        futures.join_next().await;
    }

    // Drop tx
    std::mem::drop(tx);

    sorted_fut.await.unwrap();
}

async fn order_parsed_messaged(
    mut chunk_result_recv: Receiver<(usize, Vec<MsgIterValue>)>,
    sorted_result_sender: Sender<Option<Vec<MsgIterValue>>>,
) -> Result<()> {
    let mut next_idx = 0;

    let mut parsed_ooo_chunks = BinaryHeap::new();

    loop {
        // If not check for any futures
        match chunk_result_recv.try_recv() {
            Ok((chunk_idx, msg_vals)) => {
                if chunk_idx == next_idx {
                    next_idx += 1;
                    sorted_result_sender.send(Some(msg_vals)).await.unwrap();
                } else {
                    parsed_ooo_chunks.push((-(chunk_idx as i64), msg_vals))
                }
            }
            Err(tokio::sync::mpsc::error::TryRecvError::Empty) => {}
            Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => {
                sorted_result_sender.send(None).await.unwrap();
                break;
            }
        }

        // Lastly peek at OOO chunks to see if they should be added
        loop {
            match parsed_ooo_chunks.peek() {
                Some((chunk_idx, _)) => {
                    if -chunk_idx as usize == next_idx {
                        let (_, msg_vals) = parsed_ooo_chunks.pop().unwrap();
                        next_idx += 1;
                        sorted_result_sender.send(Some(msg_vals)).await.unwrap();
                    } else {
                        break;
                    }
                }
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
        let chunk_bytes = c
            .decompress(bag.cursor.read_chunk(data_pos).await.unwrap())
            .unwrap();

        ChunkData::try_from_bytes_with_time_check(chunk_bytes, start, end).unwrap()
    } else {
        return Err(anyhow::Error::new(RosError::InvalidRecord(
            "Bad Record type detected. Expected Chunk.",
        )));
    };

    let mut message_vals = Vec::with_capacity(chunk_data.message_datas.len());
    for md in chunk_data.message_datas {
        let msg_val = match con_to_msg
            .get(&md._conn)
            .unwrap()
            .try_parse(&md.data.unwrap())
        {
            Ok((_, FieldValue::Msg(msg))) => msg,
            _ => {
                return Err(anyhow::Error::new(RosError::InvalidRecord(
                    "MessageData did not contain a parsable Value",
                )));
            }
        };
        message_vals.push((md._time, md._conn, msg_val));
    }

    tx.send((chunk_idx, message_vals)).await.unwrap();

    Ok(())
}

impl BagMessageIterator {
    pub(crate) fn new(bag: Bag, meta: Meta, start: u64, end: u64, chunk_infos: Vec<ChunkInfo>, config: BagMessageIteratorConfig) -> Self {
        let con_to_msg = meta.borrow_connection_to_id_message();

        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(8)
            .enable_time()
            .enable_io()
            .build()
            .unwrap();

        let (message_sender, message_reader) = tokio::sync::mpsc::channel(10);
        runtime.spawn(start_parse_msgs(
            bag,
            chunk_infos,
            con_to_msg.clone(),
            start,
            end,
            message_sender,
        ));

        let s = BagMessageIterator {
            _runtime: runtime,
            message_reader,
            msg_queue: VecDeque::new(),
            config
        };

        s
    }
}

impl Iterator for BagMessageIterator {
    type Item = MsgIterValue;

    fn next(&mut self) -> Option<Self::Item> {
        match self.msg_queue.pop_front() {
            Some(msg) => Some(msg),
            None => match self.message_reader.blocking_recv() {
                Some(Some(msgs)) => {
                    self.msg_queue.append(&mut msgs.into());
                    Some(self.msg_queue.pop_front().unwrap())
                }
                Some(None) | None => None,
            },
        }
    }
}
