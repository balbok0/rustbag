use ros_msg::msg_value::MsgValue;

pub(crate) const VERSION_STRING: &str = "#ROSBAG V2.0\n";
pub(crate) const VERSION_LEN: usize = VERSION_STRING.len() as usize;

pub type MsgIterValue = (u64, u32, MsgValue);
