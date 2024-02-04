//! SuperBlock/DiskInode/DirEntry
use super::{BlockDevice, BLOCK_SZ, get_block_cache};

use alloc::vec::Vec;
use alloc::sync::Arc;
use core::fmt::{Debug, Formatter, Result};

/* Some constants */
// Magic number for sanity check
const TFS_MAGIC: u32 = 0x3b800001;
// Inode related block numbers
const INODE_DIRECT_COUNT: usize = 28;
const INODE_INDIRECT1_COUNT: usize = BLOCK_SZ / 4;
const INODE_INDIRECT2_COUNT: usize = INODE_INDIRECT1_COUNT * INODE_INDIRECT1_COUNT;
const DIRECT_BOUND: usize = INODE_DIRECT_COUNT;
const INDIRECT1_BOUND: usize = DIRECT_BOUND + INODE_INDIRECT1_COUNT;
const INDIRECT2_BOUND: usize = INDIRECT1_BOUND + INODE_INDIRECT2_COUNT;
// The max length of dir_entry name
const NAME_LENGTH_LIMIT: usize = 27;
pub const DIR_ENTRY_SZ: usize = 32; // 27 + 1 + 4

/**
    [SuperBlock_Description]:
    Filesystem legitimacy checks are provided in the form of magic numbers,
    and the location of other contiguous areas can also be located.
*/
#[repr(C)]
pub struct SuperBlock {
    magic: u32,
    pub total_blocks: u32,
    pub inode_bitmap_blocks: u32,
    pub inode_area_blocks: u32,
    pub data_bitmap_blocks: u32,
    pub data_area_blocks: u32,
}

impl Debug for SuperBlock {
    fn fmt(&self, f: &mut Formatter) -> Result {
        f.debug_struct("SuperBlock")
            .field("total_blocks", &self.total_blocks)
            .field("inode_bitmap_blocks", &self.inode_bitmap_blocks)
            .field("inode_area_blocks", &self.inode_area_blocks)
            .field("data_bitmap_blocks", &self.data_bitmap_blocks)
            .field("data_area_blocks", &self.data_area_blocks)
            .finish()
    }
}

impl SuperBlock {
    pub fn initialize(
        &mut self, total_blocks: u32,
        inode_bitmap_blocks: u32, inode_area_blocks: u32,
        data_bitmap_blocks: u32, data_area_blocks: u32,
    ) {
        *self = Self {
            magic: TFS_MAGIC,
            total_blocks,
            inode_bitmap_blocks, inode_area_blocks,
            data_bitmap_blocks, data_area_blocks,
        }
    }

    pub fn is_valid(&self) -> bool {
        self.magic == TFS_MAGIC
    }
}

/**
    [DiskInode_Description]:
    Each file/directory is stored as a DiskInode on disk,
    It contains metadata about files/directories.
*/
#[derive(PartialEq)]
pub enum DiskInodeType {
    File,
    Directory,
}

type IndirectBlock = [u32; BLOCK_SZ / 4];
type DataBlock = [u8; BLOCK_SZ];

#[repr(C)]
pub struct DiskInode {
    //file size
    pub size: u32,
    //index
    pub direct: [u32; INODE_DIRECT_COUNT],
    pub indirect1: u32,
    pub indirecr2: u32,
    //disk_inode type
    type_: DiskInodeType,
}

/* Some core methods */
impl DiskInode {
    pub fn initialize(&mut self, type_: DiskInodeType) {
        self.size = 0;
        self.direct.iter_mut().for_each(|v| *v = 0);
        self.indirect1 = 0;
        self.indirecr2 = 0;
        self.type_ = type_;
    }
    ///Increase the size of current disk_inode
    ///[direct] => [indirect1] => [indirect2]
    pub fn increase_size(
        &mut self,
        new_size: u32,
        new_blocks: Vec<u32>,
        block_device: &Arc<dyn BlockDevice>
    ) {
        let mut current_blocks = self.data_blocks();
        self.size = new_size;
        let mut target_blocks = self.data_blocks();
        let mut new_blocks_iter = new_blocks.into_iter();
        //fill direct
        while current_blocks < target_blocks.min(INODE_DIRECT_COUNT as u32) {
            self.direct[current_blocks as usize] = new_blocks_iter.next().unwrap();
            current_blocks += 1;
        }
        //alloc and fill indirect1
        if target_blocks > INODE_DIRECT_COUNT as u32 {
            //indirect1 is not allocateds now
            if current_blocks == INODE_DIRECT_COUNT as u32 {
                self.indirect1 = new_blocks_iter.next().unwrap();
            }
            current_blocks -= INODE_DIRECT_COUNT as u32;
            target_blocks -= INODE_DIRECT_COUNT as u32;
        } else {
            return;
        }
        get_block_cache(self.indirect1 as usize, Arc::clone(block_device))
            .lock()
            .modify(0, |indirect1_block: &mut IndirectBlock| {
                while current_blocks < target_blocks.min(INODE_INDIRECT1_COUNT as u32) {
                    indirect1_block[current_blocks as usize] = new_blocks_iter.next().unwrap();
                    current_blocks += 1; 
                } 
            });
        //alloc and fill indirect2
        if target_blocks > INODE_INDIRECT1_COUNT as u32 {
            if current_blocks == INODE_INDIRECT1_COUNT as u32 {
                self.indirecr2 = new_blocks_iter.next().unwrap();
            }
            current_blocks -= INODE_INDIRECT1_COUNT as u32;
            target_blocks -= INODE_INDIRECT2_COUNT as u32;
        } else {
            return;
        }
        let mut a0 = current_blocks as usize / INODE_INDIRECT1_COUNT;
        let mut b0 = current_blocks as usize % INODE_INDIRECT2_COUNT;
        let a1 = target_blocks as usize / INODE_INDIRECT1_COUNT;
        let b1 = target_blocks as usize % INODE_INDIRECT1_COUNT;
        get_block_cache(self.indirecr2 as usize, Arc::clone(block_device))
            .lock()
            .modify(0, |indirect2_block: &mut IndirectBlock| {
                while a0 < a1 || (a0 == a1 && b0 < b1) {
                    if b0 == 0 {
                        indirect2_block[a0] = new_blocks_iter.next().unwrap();
                    }
                    //continue to fill indirect1_block
                    get_block_cache(indirect2_block[0] as usize, Arc::clone(block_device))
                        .lock()
                        .modify(0, |indirect1_block: &mut IndirectBlock| {
                            indirect1_block[b0] = new_blocks_iter.next().unwrap(); 
                        });
                    //move b0
                    b0 += 1;
                    if b0 == INODE_INDIRECT1_COUNT {
                        b0 = 0;
                        a0 += 1;
                    }
                }
            });
    }
    ///Clear size to zero and return blocks that should be deallocated
    ///We will clear the block contents to zero later
    pub fn  clear_size(&mut self, block_device: &Arc<dyn BlockDevice>)
        -> Vec<u32>
    {
        let mut v: Vec<u32> = Vec::new();
        let mut current_blocks = self.data_blocks() as usize;
        let mut cleared_blocks = 0usize;
        self.size = 0;
        //direct => data_block
        while cleared_blocks < current_blocks.min(INODE_DIRECT_COUNT) {
            v.push(self.direct[cleared_blocks]);
            self.direct[cleared_blocks] = 0;
            cleared_blocks += 1;
        }
        //indirect1 => indirect1_block/data_block
        if current_blocks > INODE_DIRECT_COUNT {
            v.push(self.indirect1);
            current_blocks -= INODE_DIRECT_COUNT;
            cleared_blocks = 0;
        } else {
            return v;
        }
        get_block_cache(self.indirect1 as usize, Arc::clone(block_device))
            .lock()
            .modify(0, |indirect1_block: &mut IndirectBlock| {
                while cleared_blocks < current_blocks.min(INODE_INDIRECT1_COUNT) {
                    v.push(indirect1_block[cleared_blocks]);
                    //indirect1_block[cleared_blocks] = 0;
                    cleared_blocks += 1;
                } 
            });
        self.indirect1 = 0;
        //indirect2 => indirect2_block/indirect1_block/data_block
        if current_blocks > INODE_INDIRECT1_COUNT {
            v.push(self.indirecr2);
            current_blocks -= INODE_INDIRECT1_COUNT;
        } else {
            return v;
        }
        assert!(current_blocks <= INODE_INDIRECT2_COUNT);
        let a0 = current_blocks / INODE_INDIRECT1_COUNT;
        let b0 = current_blocks % INODE_INDIRECT1_COUNT;
        get_block_cache(self.indirecr2 as usize, Arc::clone(block_device))
            .lock()
            .modify(0, |indirect2_block: &mut IndirectBlock| {
                 for indirect1 in indirect2_block.iter_mut().take(a0) {
                    v.push(*indirect1);
                    get_block_cache(*indirect1 as usize, Arc::clone(block_device))
                        .lock()
                        .modify(0, |indirect1_block: &mut IndirectBlock| {
                            for block_id in indirect1_block.iter() {
                                v.push(*block_id);
                            } 
                        });
                 }
                 if b0 > 0 {
                    v.push(indirect2_block[a0]);
                    get_block_cache(indirect2_block[a0] as usize, Arc::clone(block_device))
                        .lock()
                        .modify(0, |indirect1_block: &mut IndirectBlock| {
                            for block_id in indirect1_block.iter().take(b0) {
                                v.push(*block_id);
                            } 
                        });
                 }
            });
        self.indirecr2 = 0;
        v
    }
    ///Read data from current disk_inode
    pub fn read_at(
        &self,
        offset: usize,
        buf: &mut [u8],
        block_device: &Arc<dyn BlockDevice>,
    ) -> usize {
        let mut start = offset;
        let end = (offset + buf.len()).min(self.size as usize);
        if start >= end {
            return 0;
        }
        //read file from offset to end by per-block
        let mut read_size = 0usize;
        let mut start_block = start / BLOCK_SZ;
        loop {
            //calculate end of current block
            let mut current_block_end_size = (start / BLOCK_SZ + 1) * BLOCK_SZ;
            current_block_end_size = current_block_end_size.min(end);
            //read and update read_size
            let current_block_read_size = current_block_end_size - start;
            let dst = &mut buf[read_size..read_size + current_block_read_size];
            get_block_cache(
                self.get_block_id(start_block as u32, block_device) as usize,
                Arc::clone(block_device)
            )
            .lock()
            .read(0, |data_block: &DataBlock| {
                let src = &data_block[start % BLOCK_SZ..start % BLOCK_SZ + current_block_read_size];
                dst.copy_from_slice(src);
            });
            read_size += current_block_read_size;
            //move to next block
            if current_block_end_size == end {
                break;
            }
            start_block += 1;
            start = current_block_end_size;
        }
        read_size
    }
    ///Write data into current disk_inode
    ///Size must be adjusted properly before call `write_at`
    pub fn write_at(
        &mut self,
        offset: usize,
        buf: &[u8],
        block_device: &Arc<dyn BlockDevice>,
    ) -> usize {
        let mut start = offset;
        let end = (offset + buf.len()).min(self.size as usize);
        assert!(start <= end);
        //write data into file from offset to end by per-block
        let mut start_block = start / BLOCK_SZ;
        let mut write_size = 0usize;
        loop {
            //calculate end of current block
            let mut current_block_end_size = (start / BLOCK_SZ + 1) * BLOCK_SZ;
            current_block_end_size = current_block_end_size.min(end);
            //write and update write_size
            let current_block_write_size = current_block_end_size - start;
            get_block_cache(
                self.get_block_id(start_block as u32, block_device) as usize,
                Arc::clone(block_device)
            )
            .lock()
            .modify(0, |data_block: &mut DataBlock| {
                let src = &buf[write_size..write_size + current_block_write_size];
                let dst = &mut data_block[start % BLOCK_SZ..start % BLOCK_SZ + current_block_write_size];
                dst.copy_from_slice(src); 
            });
            write_size += current_block_write_size;
            //move to next block
            if current_block_end_size == end {
                break;
            }
            start_block += 1;
            start = current_block_end_size;
        }
        write_size
    }
}

/* Some helper methods  */
impl DiskInode {
    pub fn is_dir(&self) -> bool {
        self.type_ == DiskInodeType::Directory
    }
    pub fn is_file(&self) -> bool {
        self.type_ == DiskInodeType::File
    }
    /// Get real global_id on block device by inner DiskInode_id
    pub fn get_block_id(&self, inner_id: u32, block_device: &Arc<dyn BlockDevice>) -> u32 {
        let inner_id = inner_id as usize;
        if inner_id < INODE_DIRECT_COUNT {
            self.direct[inner_id]
        } else if inner_id < INDIRECT1_BOUND {
            get_block_cache(
                self.indirect1 as usize,
                Arc::clone(block_device)
            )
            .lock().read(0, |indirect_block: &IndirectBlock| {
                indirect_block[inner_id - INODE_DIRECT_COUNT]
            })
        } else {
            // this is inner_id for indirect2
            let indirect2_inner_id = inner_id - INDIRECT1_BOUND;
            // find the first-level index block in which the block_id is located
            let indirect1 = get_block_cache(
                self.indirecr2 as usize,
                Arc::clone(block_device)
            )
            .lock().read(0, |indirect2_block: &IndirectBlock| {
                indirect2_block[indirect2_inner_id / INODE_INDIRECT1_COUNT]
            });
            // the block_id is found by means of a first-level index block combined with an offset
            get_block_cache(
                indirect1 as usize,
                Arc::clone(block_device)
            )
            .lock().read(0, |indirect1_block: &IndirectBlock| {
                indirect1_block[indirect2_inner_id % INODE_INDIRECT1_COUNT]
            })
        }
    }
    /*
        The following methods is used to determine how many additional blocks
        are needed when capacity is expanded.

        Possible chains of function calls:
        [(vfs)Inode::write_at(offset, buf)]
                +-> [(vfs)Inode::increase_size]
                        +-> [DiskInode::block_num_needed]
                                +-> [DiskInode::increase_size] 
    */
    fn _data_blocks(size: u32) -> u32 {
        (size + BLOCK_SZ as u32 - 1) / BLOCK_SZ as u32
    }
    pub fn data_blocks(&self) -> u32 {
        Self::_data_blocks(self.size)
    }
    pub fn total_blocks(size: u32) -> u32 {
        let data_blocks = Self::_data_blocks(size) as usize;
        let mut total = data_blocks as usize;
        //indirect1
        if data_blocks > INODE_DIRECT_COUNT {
            total += 1;
        }
        //indirect2
        if data_blocks > INDIRECT1_BOUND {
            total += 1;
            total += (data_blocks - INDIRECT1_BOUND + INODE_INDIRECT1_COUNT - 1) / INODE_INDIRECT1_COUNT;
        }
        total as u32
    }
    pub fn blocks_num_needed(&self, new_size: u32) -> u32 {
        assert!(new_size >= self.size);
        Self::total_blocks(new_size) - Self::total_blocks(self.size)
    }
}

/** 
    [DirEntry_Description]:
    The contents of directories need to follow a special format. In our implementation,
    it can be viewed as a sequence of directory entries, each of which is a tuple.
*/
#[repr(C)]
pub struct DirEntry {
    name: [u8; NAME_LENGTH_LIMIT + 1], // '\0'
    inode_number: u32,
}

impl DirEntry {
    pub fn empty() -> Self {
        Self {
            name: [0u8; NAME_LENGTH_LIMIT + 1],
            inode_number: 0,
        }
    }

    pub fn new(name: &str, inode_number: u32) -> Self {
        let mut name_bytes = [0u8; NAME_LENGTH_LIMIT + 1];
        name_bytes[..name.len()].copy_from_slice(name.as_bytes());
        Self {
            name: name_bytes,
            inode_number,
        }
    }

    pub fn name(&self) -> &str {
        let len = (0usize..).find(|i| self.name[*i] == 0).unwrap();
        core::str::from_utf8(&self.name[..len]).unwrap()
    }

    pub fn inode_number(&self) -> u32 {
        self.inode_number
    }

    /** Serialize `DirEntry(self)` into bytes/mutable bytes  */
    pub fn as_bytes(&self) -> &[u8] {
        unsafe {
            core::slice::from_raw_parts(self as *const _ as usize as *const u8, DIR_ENTRY_SZ)
        }
    }
    pub fn as_bytes_mut(&mut self) -> &mut [u8] {
        unsafe {
            core::slice::from_raw_parts_mut(self as *mut _ as usize as *mut u8, DIR_ENTRY_SZ)
        }
    }
}