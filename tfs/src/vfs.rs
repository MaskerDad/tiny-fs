/*!
    Virtual file system, which provides a file operation interface
    to shield the differences of different file systems.
*/
use crate::block_cache;

use super::{
    block_cache_sync_all, get_block_cache,
    DiskInode, DiskInodeType, DirEntry,
    TinyFileSystem,
    BlockDevice,
    DIR_ENTRY_SZ,
};

use alloc::string::String;
use alloc::sync::Arc;
use alloc::vec::Vec;
use spin::{Mutex, MutexGuard};
///Virtual filesystem layer over tiny-fs
pub struct Inode{
    block_id: usize,
    offset: usize,
    fs: Arc<Mutex<TinyFileSystem>>,
    block_device: Arc<dyn BlockDevice>,
}

/*
    tiny-fs users will support file-related operations
    using the following methods.
*/
impl Inode {
    ///Create inode by name
    
    ///List inodes and return name vector
    
    ///Read data from current inode
    
    ///Write data to current inode
    
    ///Clear the data in current inode
    
}

/* tiny-fs users tend not to use the following methods directly */
impl Inode {
    ///Create a vfs inode
    pub fn new(
        block_id: u32,
        offset: usize,
        fs: Arc<Mutex<TinyFileSystem>>,
        block_device: Arc<dyn BlockDevice>,
    ) -> Self {
        Self {
            block_id: block_id as usize,
            offset,
            fs,
            block_device,
        }
    }
    ///Read disk_inode directly by vfs inode
    
    ///Modify disk_inode directly by vfs inode
    
    ///Find inode by name
    
    ///Increase the size of disk_inode by vfs inode
    
}