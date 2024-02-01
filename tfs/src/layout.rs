use super::{BlockDevice, BLOCK_SZ, get_block_cache};

use alloc::vec::Vec;
use alloc::sync::Arc;
use core::fmt::{Debug, Formatter, Result};
use std::intrinsics::saturating_add;

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
const DIR_ENTRY_SZ: usize = 32; // 27 + 1 + 4

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
    // file size
    pub size: u32,
    
    pub direct: [u32; INODE_DIRECT_COUNT],
    pub indirect1: u32,
    pub indirecr2: u32,
    type_: DiskInodeType,
}

impl DiskInode {
    pub fn initialize(&mut self, type_: DiskInodeType) {
        self.size = 0;
        self.direct.iter_mut().for_each(|v| *v = 0);
        self.indirect1 = 0;
        self.indirecr2 = 0;
        self.type_ = type_;
    }

    pub fn is_dir(&self) -> bool {
        self.type_ == DiskInodeType::Directory
    }

    pub fn is_file(&self) -> bool {
        self.type_ == DiskInodeType::File
    }

    //TODO
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

