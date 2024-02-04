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
    let block_file = Arc::new(BlockFile(Mutex::new(
        {
            let f = OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .open("target/fs.img")?;
            f.set_len(8192 * 512).unwrap();
            f
        }
    )));
    //TinyFileSystem::create(block_file.clone(), 4096, 1);
    let tfs = TinyFileSystem::open(block_file.clone());
    let root_inode = TinyFileSystem::root_inode(&tfs);
    //create file test
    root_inode.create("file_a");
    root_inode.create("file_b");
    for name in root_inode.ls() {
        println!("{}", name);
    }
    //write file test
    let test_str = "hello, tiny-fs!";
    let file_a = root_inode.find("file_a").unwrap();
    file_a.write_at(0, test_str.as_bytes());
    let mut buf = [0u8; 512];
    let len = file_a.read_at(0, &mut buf);
    assert_eq!(test_str, core::str::from_utf8(&bufs[..len]).unwrap());
    
    //random string test
    let mut random_str_test = |len: usize| {
        use rand;
        
        file_a.clear();
        assert_eq!(file_a.read_at(0, &mut buf), 0);
        let mut str_random = String::new();
        //create a random string and write into the file_a
        for _ in 0..len {
            str_random.push(char::from('0' as u8 + rand::random::<u8>() % 10));
        }
        file_a.write_at(0, str_random.as_bytes());
        //file_a read test
        let mut read_str = String::new();
        //read one part at a time
        let mut read_buf = [0u8; 10];
        let mut offset = 0usize;
        loop {
            let len = file_a.read_at(offset, &mut read_buf);
            if len == 0 {
                break;
            }
            offset += len;
            read_str.push(core::str::from_utf8(&read_buf[..len]).unwrap());
        }
        assert_eq!(str_random, read_str);
    };
    
    random_str_test(4 * BLOCK_SZ);
    random_str_test(8 * BLOCK_SZ + BLOCK_SZ / 2);
    random_str_test(100 * BLOCK_SZ);
    random_str_test(70 * BLOCK_SZ + BLOCK_SZ / 7);
    random_str_test((12 + 128) * BLOCK_SZ);
    random_str_test(400 * BLOCK_SZ);
    random_str_test(1000 * BLOCK_SZ);
    random_str_test(2000 * BLOCK_SZ);
    
    Ok(())
}