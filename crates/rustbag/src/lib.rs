#![warn(missing_docs)]
//! RustBag is a Rust-based reader of ROSBag files, with Python bindings.
//! The focus of this crate is to leverage object_store crate and enable reading directly from S3-compatible endpoints.
//!
//! Please note that the crate is still in alpha. Bugs are currently being resolved, and functionality is still being added.
//!
//! [Python documentation](https://balbok0.github.io/rustbag/)


pub mod bag;
pub mod bag_msg_iterator;
mod constants;
mod cursor;
mod error;
mod iterators;
mod meta;
mod records;
mod utils;

pub use bag::Bag;
pub use bag_msg_iterator::BagMessageIterator;
