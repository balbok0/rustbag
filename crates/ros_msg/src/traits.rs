use anyhow::Result;

use crate::msg_value::FieldValue;

pub(crate) trait MaybeSized {
    fn known_size(&self) -> Option<usize>;
}

pub trait ParseBytes {
    fn try_parse(&self, bytes: &[u8]) -> Result<(usize, FieldValue)>;
}