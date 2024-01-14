pub(crate) const VERSION_STRING: &str = "#ROSBAG V2.0\n";
pub(crate) const VERSION_LEN: usize = VERSION_STRING.len() as usize;
pub(crate) const ROSBAG_HEADER_SIZE: usize = 4096;
pub(crate) const ROSBAG_HEADER_OP: u8 = 0x03;
