# lwext4-rs

A crate for interfacing with [lwext4](https://github.com/gkostka/lwext4) from Rust.

## Details
You can find the details of the interface in [interface.md](interface.md).

## Usage
```rust
fn main(){
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .open("ext_images/ext_image")
        .unwrap();
    let mut config = BlockDeviceConfig::default();

    let bs: u64 = 512;
    config.block_size = bs as u32;
    config.part_size = file.metadata().unwrap().len();
    config.part_offset = 0;
    config.block_count = config.part_size / bs;

    println!("config: {:#x?}", config);

    set_debug_mask(DebugFlags::ALL);
    let blk = DefaultInterface::new_device(file, config);
    let register_handler = RegisterHandle::register(blk, "ext4fs".to_string()).unwrap();
    let mount_handler =
        MountHandle::mount(register_handler, "/mp/".to_string(), true, false).unwrap();
    let fs = FileSystem::new(mount_handler).unwrap();

    let stats = fs.mount_handle().stats().unwrap();
    println!("stats: {:#x?}", stats);

    let read_dir = fs.readdir("/mp/").unwrap();
    for entry in read_dir {
        println!("{:?}", entry);
    }
}
```

## Reference
[lwext4 (C)](https://github.com/gkostka/lwext4)

[lwext4 (rust)](https://github.com/djdisodo/lwext4)