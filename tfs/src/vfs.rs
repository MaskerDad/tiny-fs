/*!
    Virtual file system, which provides a file operation interface
    to shield the differences of different file systems.
*/
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
    /* 
        block_id and offset determine the position of
        the corresponding disk_inode on the block device.
    */
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
    pub fn create(&self, name: &str) -> Option<Arc<Inode>> {
        let mut fs = self.fs.lock();
        //find inode by name
        if self.read_disk_inode(|root_inode: &DiskInode| {
            assert!(root_inode.is_dir());
            //has the file been created?
            self.find_inode_id(name, root_inode)
        }).is_some() {
            //no new inode need be created
            return None;
        }
        //new inode need be created
        /* initialize new_inode */
        let new_inode_bit = fs.alloc_inode();
        let (new_inode_block_id, new_inode_offset)
            = fs.get_disk_inode_pos(new_inode_bit);
        get_block_cache(
            new_inode_block_id as usize,
            Arc::clone(&self.block_device)
        ).lock()
        .modify(new_inode_offset, |new_inode: &mut DiskInode| {
            new_inode.initialize(DiskInodeType::File); 
        });
        /* update root_inode to contains new_inode */
        self.modify_disk_inode(|root_inode| {
            //apend dir_entry in the root_inode directory
            //update meta_data
            let file_count = (root_inode.size as usize) / DIR_ENTRY_SZ;
            let new_size = (file_count + 1) * DIR_ENTRY_SZ;
            //increase size
            self.increase_size(new_size as u32, root_inode, &mut fs);
            //write dir_entry
            let dirent = DirEntry::new(name, new_inode_bit);
            root_inode.write_at(
                file_count * DIR_ENTRY_SZ,
                dirent.as_bytes(),
                &self.block_device
            );
        });
        /* create and return new_inode */
        let (block_id, offset) = fs.get_disk_inode_pos(new_inode_bit);
        Some(Arc::new(Inode::new(
            block_id,
            offset,
            self.fs.clone(),
            self.block_device.clone()
        )))
    }
    ///Find inode by name
    pub fn find(&self, name: &str) -> Option<Arc<Inode>> {
        let fs = self.fs.lock();
        self.read_disk_inode(|disk_inode| {
            self.find_inode_id(name, disk_inode).map(|inode_bit| {
                let (block_id, offset) = fs.get_disk_inode_pos(inode_bit);
                Arc::new(Self::new(
                    block_id,
                    offset,
                    self.fs.clone(),
                    self.block_device.clone()
                ))
            })
        })
    }
    ///List inodes and return name vector
    pub fn ls(&self) -> Vec<String> {
        let _fs = self.fs.lock();
        self.read_disk_inode(|disk_inode| {
            let file_count = (disk_inode.size as usize) / DIR_ENTRY_SZ;
            let mut v: Vec<String> = Vec::new();
            for i in 0..file_count {
                let mut dir_entry = DirEntry::empty();
                assert_eq!(
                    disk_inode.read_at(
                        DIR_ENTRY_SZ * i,
                        dir_entry.as_bytes_mut(),
                        &self.block_device
                    ),
                    DIR_ENTRY_SZ
                );
                v.push(String::from(dir_entry.name()));
            }
            v
        })
    }
    ///Read data from current inode
    pub fn read_at(&self, offset: usize, buf: &mut [u8]) -> usize {
        let _fs = self.fs.lock();
        self.read_disk_inode(|disk_inode| {
            disk_inode.read_at(offset, buf, &self.block_device)
        })
    }
    ///Write data to current inode
    pub fn write_at(&self, offset: usize, buf: &[u8]) -> usize {
        let mut fs = self.fs.lock();
        let write_size = self.modify_disk_inode(|disk_inode| {
            self.increase_size(
                (offset + buf.len()) as u32,
                disk_inode, &mut fs
            );
            disk_inode.write_at(offset, buf, &self.block_device)
        });
        block_cache_sync_all();
        write_size
    }
    ///Clear the data in current inode
    pub fn clear(&self) {
        let mut fs = self.fs.lock();
        self.modify_disk_inode(|disk_inode| {
            let size =disk_inode.size;
            let data_blocks_dealloc = disk_inode.clear_size(&self.block_device);
            //dealloc_blocks_num == disk_inode.total_blocks?
            assert!(
                data_blocks_dealloc.len() ==
                DiskInode::total_blocks(size) as usize
            );
            for block_id in data_blocks_dealloc.into_iter() {
                fs.dealloc_data(block_id);
            }
        });
        block_cache_sync_all();
    }
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
    ///Read disk_inode directly with f by vfs inode
    fn read_disk_inode<V>(
        &self,
        f: impl FnOnce(&DiskInode) -> V
    ) -> V {
        get_block_cache(self.block_id, Arc::clone(&self.block_device))
            .lock()
            .read(self.offset, f)
    }
    ///Modify disk_inode directly with f by vfs inode
    fn modify_disk_inode<V>(
        &self,
        f: impl FnOnce(&mut DiskInode) -> V
    ) -> V {
        get_block_cache(self.block_id, Arc::clone(&self.block_device))
            .lock()
            .modify(self.offset, f)
    }
    ///Increase the size of disk_inode by vfs inode
    fn increase_size(
        &self,
        new_size: u32,
        disk_inode: &mut DiskInode,
        fs: &mut MutexGuard<TinyFileSystem>,
    ) {
        if new_size < disk_inode.size {
            return;
        }
        let blocks_needed = disk_inode.blocks_num_needed(new_size);
        let mut v: Vec<u32> = Vec::new();
        for _ in 0..blocks_needed {
            v.push(fs.alloc_data());
        }
        //move to DiskInode layer to complete increase_size
        disk_inode.increase_size(new_size, v, &self.block_device);
    }
    ///Find inode under disk_inode by name
    fn find_inode_id(&self, name: &str, disk_inode: &DiskInode)
        -> Option <u32>
    {
        //assert it is a directory
        assert!(disk_inode.is_dir());
        let file_count = (disk_inode.size as usize) / DIR_ENTRY_SZ;
        let mut dir_entry = DirEntry::empty();
        for i in 0..file_count {
            assert_eq!(
                disk_inode.read_at(
                    DIR_ENTRY_SZ *i,
                    dir_entry.as_bytes_mut(),
                    &self.block_device
                ),
                DIR_ENTRY_SZ
            );
            if dir_entry.name() == name {
                return Some(dir_entry.inode_number() as u32);
            }
        }
        None
    }
}