/*!
    The DiskManager is used to manage the various disk data structures
    and calls methods to adjust the filesystem layout.
*/
use super::{
    block_cache_sync_all, get_block_cache,
    SuperBlock, Bitmap, DiskInode, DiskInodeType,
    BlockDevice,
    BLOCK_SZ,
};

use alloc::sync::Arc;
use spin::Mutex;
///An tiny filesystem on block
pub struct TinyFileSystem {
    ///Real device that implemented BlockDevice
    pub block_device: Arc<dyn BlockDevice>,
    ///Inode bitmap
    pub inode_bitmap: Bitmap,
    ///Data bitmap
    pub data_bitmap: Bitmap,
    inode_area_start_block: u32,
    data_area_start_block: u32,
}

/* core methods */
impl TinyFileSystem {
    ///Create a filesystem on block device
    pub fn create(
        block_device: Arc<dyn BlockDevice>,
        total_blocks: u32,
        inode_bitmap_blocks: u32,
    ) -> Arc<Mutex<Self>> {
        
    }
    ///Open a block device as a filesystem
    ///This function is often more commonly used than `create`
    pub fn open(block_device: Arc<dyn BlockDevice>) -> Arc<Mutex<Self>> {
        
    }
    ///Allocate a new inode
    pub fn alloc_inode(&mut self) -> u32 {
        
    }
    ///Allocate a data block
    pub fn alloc_data(&mut self) -> u32 {
        
    }
    ///Deallocate a data block
    pub fn dealloc_data(&mut self, block_id: u32) {
        
    }
    ///Get the root inode of the filesystem
    pub fn root_inode(tfs: &Arc<Mutex<Self>>) -> Inode {
        
    } 
        
}

/* helper methods */
impl TinyFileSystem {
    ///Get global data_block_id by bitmap_id
    pub fn get_data_block_id(&self, bitmap_id: u32) -> u32 {

    }
    ///Get inode position by bitmap_id
    pub fn get_disk_inode_pos(&self, bitmap_id: u32) -> (u32, usize) {
        
    }
}