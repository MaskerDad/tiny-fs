use crate::block_dev;

use super::{BlockDevice, BLOCK_SZ};

use lazy_static::*;
use alloc::collections::VecDeque;
use alloc::sync::Arc;
use spin::Mutex;


/// BlockCache mapped on block device
pub struct BlockCache {
    /// cache data
    cache: [u8; BLOCK_SZ],
    /// block id
    block_id: usize,
    /// block_device that implement BlockDevice trait
    block_device: Arc<dyn BlockDevice>,
    /// whether dirty
    modified: bool,
}

impl BlockCache {
    /// Load a new BlockCache from block device
    pub fn new(block_id: usize, block_device: Arc<dyn BlockDevice>) -> Self {
        let mut cache = [0u8; BLOCK_SZ];
        block_device.read_block(block_id, &mut cache);
        BlockCache {
            cache,
            block_id,
            block_device,
            modified: false,
        }
    }

    fn addr_of_offset(&self, offset: usize) -> usize {
        &self.cache[offset] as *const u8 as usize
    }

    pub fn obtain_ref<T>(&self, offset: usize) -> &T
    where
        T: Sized,
    {
        let type_size = core::mem::size_of::<T>();
        assert!(offset + type_size <= BLOCK_SZ);
        let addr_offset = self.addr_of_offset(offset);
        unsafe {
            &*(addr_offset as *const T)
        }
    }

    pub fn obtain_mut<T>(&mut self, offset: usize) -> &mut T
    where
        T: Sized,
    {
        let type_size = core::mem::size_of::<T>();
        assert!(offset + type_size <= BLOCK_SZ);
        let addr_offset = self.addr_of_offset(offset);
        unsafe {
            &mut *(addr_offset as *mut T)
        }
    }

    pub fn read<T, V>(&self, offset: usize, f: impl FnOnce(&T) -> V) -> V {
        f(self.obtain_ref(offset))
    }

    pub fn modify<T, V>(&mut self, offset: usize, f: impl FnOnce(&mut T) -> V) -> V {
        f(self.obtain_mut(offset))
    }

    pub fn sync(&mut self) {
        if self.modified {
            self.modified = false;
            self.block_device.write_block(self.block_id, &self.cache);
        }
    }    
}

impl Drop for BlockCache {
    fn drop(&mut self) {
        self.sync();
    }
}

/* BlockCache-Manager */
const BLOCK_CACHE_SIZE: usize = 16;

pub struct BlockCacheManager {
    // (block_id, block_cache)
    queue: VecDeque<(usize, Arc<Mutex<BlockCache>>)>,
}

impl BlockCacheManager {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
        }
    }

    pub fn get_block_cache(&mut self, block_id: usize, block_device: Arc<dyn BlockDevice>)
        -> Arc<Mutex<BlockCache>>
    {
        if let Some(pair) =
            self.queue.iter().find(|pair| pair.0 == block_id)
        {
            Arc::clone(&pair.1)
        } else {
            if self.queue.len() == BLOCK_CACHE_SIZE {
                // Delete a block_cache that is not used elsewhere
                if let Some((idx, _)) = self.queue
                    .iter()
                    .enumerate()
                    .find(|(_, pair)| Arc::strong_count(&pair.1) == 1)
                {
                    self.queue.drain(idx..=idx);
                } else {
                    panic!("Run out of BlockCache!");
                }
            }
            let block_cache = Arc::new(Mutex::new(BlockCache::new(
                block_id,
                Arc::clone(&block_device),
            )));
            self.queue.push_back((block_id, Arc::clone(&block_cache)));
            block_cache
        }
    }
}

lazy_static! {
    pub static ref BLOCK_CACHE_MANAGER: Mutex<BlockCacheManager> =
        Mutex::new(BlockCacheManager::new());
}

pub fn get_block_cache(block_id: usize, block_device: Arc<dyn BlockDevice>)
    -> Arc<Mutex<BlockCache>>
{
    BLOCK_CACHE_MANAGER
        .lock()
        .get_block_cache(block_id, block_device)
}

pub fn block_cache_sync_all() {
    let manager = BLOCK_CACHE_MANAGER.lock();
    for (_, cache) in manager.queue.iter() {
        cache.lock().sync();
    }
}