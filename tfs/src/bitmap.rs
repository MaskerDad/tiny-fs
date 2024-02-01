//! Bitmap for {inode_bitmap/data_bitmap}
use super::{get_block_cache, BlockDevice, BLOCK_SZ};

use alloc::sync::Arc;

type BitmapBlock = [u64; 64];

const BLOCK_BITS: usize = BLOCK_SZ * 8;

/// Area for inode/data_bitmap
pub struct Bitmap {
    start_block_id: usize, 
    blocks: usize,
}

impl Bitmap {
    pub fn new(start_block_id: usize, blocks: usize) -> Self {
        Self {
            start_block_id,
            blocks,
        }
    }
    /** 
        Allocate a new block from a block device:
            *return: not global_id on block device, is the inner_id of bitmap
    */    
    pub fn alloc(&self, block_device: &Arc<dyn BlockDevice>) -> Option<usize> {
        for inner_id in 0..self.blocks {
            let pos = get_block_cache(
                inner_id + self.start_block_id as usize,
                Arc::clone(block_device)
            )
            .lock()
            .modify(0, |bitmap_block: &mut BitmapBlock| {
                if let Some((bits64_pos, inner_pos)) = bitmap_block
                    .iter()
                    .enumerate()
                    .find(|(_, bits64)| **bits64 != u64::MAX)
                    .map(|(bits64_pos, bits64)| (bits64_pos, bits64.trailing_ones() as usize))
                {
                    // set 1 to allocate block
                    bitmap_block[bits64_pos] |= 1u64 << inner_pos;
                    Some(inner_id * BLOCK_BITS + bits64_pos * 64 + inner_pos)
                } else {
                    None
                }
            });
            
            if pos.is_some() {
                return pos;
            }
        }
        None
    }
    /// Deallocate a block
    pub fn dealloc(&self, block_device: &Arc<dyn BlockDevice>, bit: usize) {
        let (block_pos, bits64_pos, inner_pos) = Self::decomposition(bit);
        get_block_cache(
            self.start_block_id + block_pos,
            Arc::clone(block_device)
        )
        .lock()
        .modify(0, |bitmap_block: &mut BitmapBlock| {
            // the bit must be allocated!
            assert!(bitmap_block[bits64_pos] & (1u64 << inner_pos) > 0);
            bitmap_block[bits64_pos] -= 1u64 << inner_pos;
        });
    }
    /// Get the max number of allocatable blocks
    pub fn maxium(&self) -> usize {
        self.blocks * BLOCK_BITS
    }
    /// Decomposition `bit_id` is used to [dealloc]
    /// (block_pos, bits64_pos, inner_pos)
    fn decomposition(mut bit: usize) -> (usize, usize, usize) {
        let block_pos = bit / BLOCK_BITS;
        bit %= BLOCK_BITS;
        (block_pos, bit / 64, bit % 64)
    }
}