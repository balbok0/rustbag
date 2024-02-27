//! Crate for describing ROS messages, and handling MSG and SRV files.
#![warn(missing_docs)]

mod data_type;
mod dynamic_msg;
mod error;
mod field_info;
mod message_path;
mod msg;
mod parse_msg;
mod rosmsg;
mod srv;
#[cfg(test)]
mod tests;
mod time;
mod value;

pub use data_type::{DataType, I8Variant, U8Variant};
pub use dynamic_msg::DynamicMsg;
pub use error::{Error, Result};
pub use field_info::{FieldCase, FieldInfo};
pub use message_path::MessagePath;
pub use msg::Msg;
pub use rosmsg::RosMsg;
pub use srv::Srv;
pub use time::{Duration, Time};
pub use value::{MessageValue, Value};
