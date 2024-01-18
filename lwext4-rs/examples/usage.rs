use lwext4_rs::{
    fs, BlockDeviceConfig, MountHandle, RegisterHandle,
    SimpleBlockDeviceInterface,
};
use std::fs::OpenOptions;

fn main() {
    env_logger::init();
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .open("ext_image")
        .unwrap();
    let mut config = BlockDeviceConfig::default();

    let bs: u64 = 512;
    config.block_size = bs as u32;
    config.part_size = file.metadata().unwrap().len();
    config.part_offset = 0;
    config.block_count = config.part_size / bs;

    println!("config: {:#x?}", config);

    let blk = SimpleBlockDeviceInterface::new_device(file, config);
    let register_handler = RegisterHandle::register(blk, "ext4fs".to_string()).unwrap();
    let _mount_handler = MountHandle::mount(register_handler, "/mp/".to_string(), false).unwrap();
    let read_dir = fs::readdir("/mp/").unwrap();
    for entry in read_dir {
        println!("{}", entry.path());
        println!("is_dir: {}", entry.file_type().is_dir());
    }
    // drop(mount_handler);
}
