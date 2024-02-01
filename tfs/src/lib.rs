//!An tiny file system isolated from the kernel
#![no_std]
#![deny(missing_docs)]

mod bitmap;
mod block_cache;
mod block_dev;
mod disk_manager;
mod layout;
mod vfs;

extern crate alloc;

pub use block_dev::BlockDevice;

/// A block size of 512-bytes
pub const BLOCK_SZ: usize = 512;