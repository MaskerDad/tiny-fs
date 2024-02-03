//! tiny-fs pack and test
use tiny_fs::{BlockDevice, TinyFileSystem, BLOCK_SZ};

use clap::{App, Arg};
use core::slice::SlicePattern;
use std::fs::{read_dir, File, OpenOptions};
use std::io::{Read, Write, Seek, SeekFrom};
use std::sync::Arc;
use std::sync::Mutex;

struct BlockFile(Mutex<File>);

impl BlockDevice for BlockFile {
    fn read_block(&self, block_id: usize, buf: &mut [u8]) {
        let mut file = self.0.lock().unwrap();
        file.seek(SeekFrom::start((block_id * BLOCK_SZ) as u64))
            .expect("Error when seeking!");
        assert_eq!(file.read(buf).unwrap(), BLOCK_SZ, "Not a complete block!");
    }

    fn write_block(&self, block_id: usize, buf: &[u8]) {
        let mut file = self.0.lock().unwrap();
        file.seek(SeekFrom::Start((block_id * BLOCK_SZ) as u64))
            .expect("Error when seeking!");
        assert_eq!(file.write(buf).unwrap(), BLOCK_SZ, "Not a complete block!");
    }
} 

fn main() {
    tiny_fs_pack().expect("Error when packing tiny-fs!");
}

fn tiny_fs_pack() -> std::io::Result<()> {
    let matches = App::new("TinyFileSystem packer")
        .arg(
            Arg::with_name("source")
                .short("s")
                .long("source")
                .takes_value(true)
                .help("Exectuable source dir(with backslash)"),
        )
        .arg(
            Arg::with_name("target")
                .short("t")
                .long("target")
                .takes_value(true)
                .help("Executable target dir(with backslash)"),
        )
        .get_matches();
    let src_path = matches.value_of("source").unwrap();
    let target_path = matches.value_of("target").unwrap();
    println!("src_path = {}", src_path);
    println!("target_path = {}", target_path);
    //create and open block_file "tfs.img"
    let block_file = Arc::new(BlockFile(Mutex::new(
        {
            let f = OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .open(format!("{}{}", target_path, "tfs.img"))?;
            f.set_len(16 * 2048 * 512).unwrap();
            f
        }
    )));
    //create tiny-fs
    let tfs = TinyFileSystem::create(block_file, 16 * 2048, 1);
    let root_inode = Arc::new(TinyFileSystem::root_inode(&tfs));
    let apps_name: Vec<_> = read_dir(src_path)
        .unwrap()
        .into_iter()
        .map(|dir_entry| {
            let mut name_with_ext = dir_entry.unwrap().file_name().into_string().unwrap();
            name_with_ext.drain(name_with_ext.find('.').unwrap()..name_with_ext.len());
            name_with_ext
        })
        .collect();
    for name in apps_name {
        //load app data from host file system
        let mut host_file = File::open(format!("{}{}", target_path, app)).unwrap();
        let mut app_data: Vec<u8> = Vec::new();
        host_file.read_to_end(&mut app_data).unwrap();
        //create file inode in tiny-fs
        let new_inode = root_inode.create(name.as_str()).unwrap();
        new_inode.write_at(0, app_data.as_slice());
    }
    Ok(())  
}

#[test]
fn tiny_fs_test() -> std::io::Result<()> {
       
}