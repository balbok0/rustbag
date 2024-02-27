use std::sync::Arc;

use byteorder::{ByteOrder, LE};
use object_store::{ObjectStore, ObjectMeta};
use bytes::Bytes;
use anyhow::Result;
use crate::error::RosError;


#[derive(Debug, Clone)]
pub(crate) struct Cursor {
    pub(crate) store: Arc<Box<dyn ObjectStore>>,
    pub(crate) meta: ObjectMeta,
}

impl Cursor {
    pub fn new(store: Arc<Box<dyn ObjectStore>>, meta: ObjectMeta) -> Self {
        Self { store, meta}
    }

    pub fn len(&self) -> usize {
        self.meta.size
    }

    pub async fn read_bytes(&self, pos: usize, n: usize) -> Result<Bytes> {
        if pos + n > self.len() {
            return Err(RosError::OutOfBounds.into());
        }
        self.store.get_range(&self.meta.location, pos..pos + n).await.map_err(anyhow::Error::new)
    }

    pub async fn read_chunk(&self, pos: usize) -> Result<Bytes> {
        let n = self.read_u32(pos).await? as usize;
        self.read_bytes(pos + 4, n).await
    }

    pub async fn read_u32(&self, pos: usize) -> Result<u32> {
        Ok(LE::read_u32(&self.store.get_range(&self.meta.location, pos..pos + 4).await?))
    }

    /*
    pub fn next_u64(&mut self) -> Result<u64, OutOfBounds> {
        Ok(LE::read_u64(self.next_bytes(4)?))
    }
    */

    pub async fn read_time(&self, pos: usize) -> Result<u64> {
        let s = self.read_u32(pos).await? as u64;
        let ns = self.read_u32(pos + 4).await? as u64;
        Ok(1_000_000_000 * s + ns)
    }
}
