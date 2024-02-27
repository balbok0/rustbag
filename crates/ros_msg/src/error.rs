use std::fmt;

/// The error type for ROS bag file reading and parsing.
#[derive(Debug)]
pub enum RosError {
    /// Invalid Value Type
    InvalidType,
    /// Invalid Length (for vectors, arrays & strings)
    InvalidLength
}


impl fmt::Display for RosError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use RosError::*;
        let s = match self {
            InvalidType => "Invalid Type".to_string(),
            InvalidLength => "Invalid Length".to_string(),
        };
        write!(f, "rosbag::Error: {}", s)
    }
}

impl std::error::Error for RosError {}


