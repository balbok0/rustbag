use std::sync::Arc;

use anyhow::Result;
use object_store::{ObjectMeta, ObjectStore};

use crate::{meta::Meta, cursor::{Cursor, self}, constants::{VERSION_LEN, VERSION_STRING}, error::RosError};

#[derive()]
pub struct Bag {
    // pub bag_meta: Meta,
    pub cursor: Cursor,
}


impl Bag {
    async fn new_from_object_store_meta(object_store: Arc<Box<dyn ObjectStore>>, object_meta: ObjectMeta) -> Self {
        let cursor = Cursor::new(object_store, object_meta);

        if let Err(e) = read_bag_header(&cursor).await {
            panic!("{}", e);
        };
        Bag {
            cursor: cursor,
        }
    }
}


// Helper Function
async fn read_bag_header(cursor: &Cursor) -> Result<()> {
    let bag_version_header = cursor.read_bytes(0, VERSION_LEN).await?;
    if bag_version_header != VERSION_STRING {
        return Err(RosError::InvalidVersion.into());
    }

    Ok(())
}