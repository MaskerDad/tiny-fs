use core::any::Any;

/// BlockDevice is implemented by outer tiny-fs user
pub trait BlockDevice: Send + Sync + Any {
    /// read data from block device by os driver
    fn read_block(&self, block_id: usize, buf: &mut [u8]);
    /// write data to block device by os driver
    fn write_block(&self, block_id: usize, buf: &[u8]);
}