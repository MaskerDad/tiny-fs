//! An tiny file system isolated from the kernel
#![no_std]
#![deny(missing_docs)]

mod bitmap;
mod block_cache;
mod block_dev;
mod tfs;
mod layout;
mod vfs;

extern crate alloc;

pub use block_dev::BlockDevice;
pub use tfs::TinyFileSystem;
pub use vfs::Inode;
use block_cache::{get_block_cache, block_cache_sync_all};
use bitmap::Bitmap;
use layout::*;
/// A block size of 512-bytes
pub const BLOCK_SZ: usize = 512;