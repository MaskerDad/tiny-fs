/*!
    The DiskManager is used to manage the various disk data structures
    and calls methods to adjust the filesystem layout.
*/
use super::{
    block_cache_sync_all, get_block_cache,
    SuperBlock, Bitmap, DiskInode, DiskInodeType,
    Inode,
    BlockDevice,
    BLOCK_SZ,
};

use alloc::sync::Arc;
use spin::Mutex;

type DataBlock = [u8; BLOCK_SZ];
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

/* create/open/root_inode */
impl TinyFileSystem {
    ///Create a filesystem on block device
    pub fn create(
        block_device: Arc<dyn BlockDevice>,
        total_blocks: u32,
        inode_bitmap_blocks: u32,
    ) -> Arc<Mutex<Self>> {
        //create bitmaps
        //calculate block_size of areas 
        let inode_bitmap = Bitmap::new(1, inode_bitmap_blocks as usize);
        let inode_num = inode_bitmap.maxium();
        let inode_area_blocks =
            ((inode_num * core::mem::size_of::<DiskInode>() + BLOCK_SZ - 1) / BLOCK_SZ) as u32;
        let inode_total_blocks = inode_bitmap_blocks + inode_area_blocks;
        let data_total_blocks = total_blocks - 1 - inode_area_blocks;
        let data_bitmap_blocks = (data_total_blocks + 4096) / 4097;
        let data_area_blocks = data_total_blocks - data_bitmap_blocks;
        let data_bitmap = Bitmap::new(
            (1 + inode_bitmap_blocks + inode_area_blocks) as usize,
            data_bitmap_blocks as usize
        );
        let inode_area_start_block = 1 + inode_bitmap_blocks;
        let data_area_start_block = 1 + inode_total_blocks + data_bitmap_blocks;
        //create tfs
        let mut tfs = Self {
            block_device: Arc::clone(&block_device),
            inode_bitmap,
            data_bitmap,
            inode_area_start_block,
            data_area_start_block,
        };
        //clear all blocks
        for i in 0..total_blocks {
            get_block_cache(i as usize, Arc::clone(&block_device))
                .lock()
                .modify(0, |data_block: &mut DataBlock| {
                   for byte in data_block.iter_mut() {
                    *byte = 0;
                   } 
                });
        }
        //initialize SuperBlock
        get_block_cache(0, Arc::clone(&block_device))
            .lock()
            .modify(0, |super_block: &mut SuperBlock| {
               super_block.initialize(
                    total_blocks,
                    inode_bitmap_blocks,
                    inode_area_blocks,
                    data_bitmap_blocks,
                    data_area_blocks
                );
            });
        //create root_inode
        assert_eq!(tfs.alloc_inode(), 0);
        let (root_inode_block_id, root_inode_offset)
            = tfs.get_disk_inode_pos(0);
        get_block_cache(
            root_inode_block_id as usize,
            Arc::clone(&block_device)
        )
        .lock()
        .modify(root_inode_offset, |disk_inode: &mut DiskInode| {
            disk_inode.initialize(DiskInodeType::Directory); 
        });
        //return tfs
        block_cache_sync_all();
        Arc::new(Mutex::new(tfs))
    }
    ///Open a block device as a filesystem
    ///This function is often more commonly used than `create`
    pub fn open(block_device: Arc<dyn BlockDevice>) -> Arc<Mutex<Self>> {
        //read super_block
        get_block_cache(0, Arc::clone(&block_device))
            .lock()
            .read(0, |super_block: &SuperBlock| {
                assert!(super_block.is_valid(), "Error loading TFS!");
                let inode_bitmap = Bitmap::new(
                    1,
                    super_block.inode_area_blocks as usize
                );
                let inode_total_blocks =
                    super_block.inode_area_blocks + super_block.inode_bitmap_blocks;
                let data_bitmap = Bitmap::new(
                    (1 + inode_total_blocks) as usize,
                    super_block.data_bitmap_blocks as usize
                );
                let inode_area_start_block = 1 + super_block.inode_bitmap_blocks;
                let data_area_start_block = 1 + inode_total_blocks + super_block.data_bitmap_blocks;
                let tfs = Self {
                    block_device,
                    inode_bitmap,
                    data_bitmap,
                    inode_area_start_block,
                    data_area_start_block,
                };
                Arc::new(Mutex::new(tfs))
            })
    }
    ///Get the root_inode of the filesystem(is not DiskInode and return Inode)
    pub fn root_inode(tfs: &Arc<Mutex<Self>>) -> Inode {
        let (block_id, offset) = tfs.lock().get_disk_inode_pos(0);
        Inode::new(
            block_id,
            offset,
            Arc::clone(tfs),
            Arc::clone(&tfs.lock().block_device),
        )
    }
}

/* allocation and get global position on block device */
impl TinyFileSystem {
    ///Allocate a new inode and return bit
    pub fn alloc_inode(&mut self) -> u32 {
        self.inode_bitmap.alloc(&self.block_device).unwrap() as u32
    }
    ///Allocate a data block and return global_id
    pub fn alloc_data(&mut self) -> u32 {
        self.data_bitmap.alloc(&self.block_device).unwrap() as u32 + self.data_area_start_block
    }
    ///Deallocate a data block by global_id
    pub fn dealloc_data(&mut self, block_id: u32) {
        get_block_cache(block_id as usize, Arc::clone(&self.block_device))
            .lock()
            .modify(0, |data_block: &mut DataBlock| {
                data_block.iter_mut().for_each(|p| {
                    *p = 0;
                })
            });
        self.data_bitmap.dealloc(
            &self.block_device,
            (block_id - self.data_area_start_block) as usize
        );
    }
    ///Get global data_block_id by bit
    pub fn get_data_block_id(&self, data_bit: u32) -> u32 {
        self.data_area_start_block + data_bit
    }
    ///Get inode position by bit
    pub fn get_disk_inode_pos(&self, inode_bit: u32) -> (u32, usize) {
        let inode_size = core::mem::size_of::<DiskInode>();
        let inodes_per_block = (BLOCK_SZ / inode_size) as u32;
        let block_id = self.inode_area_start_block + inode_bit / inodes_per_block;
        (
            block_id,
            (inode_bit % inodes_per_block) as usize * inode_size,
        )
    }
}