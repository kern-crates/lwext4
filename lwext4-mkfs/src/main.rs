use clap::{arg, command, value_parser};
use lwext4_rs::FsType::{Ext2, Ext3, Ext4};
use lwext4_rs::{BlockDeviceConfig, DefaultInterface, FsBuilder};
use std::fs::OpenOptions;
use std::path::PathBuf;

fn main() {
    let matches = command!()
        .arg(
            arg!(-f --file <FILE> "img file path")
                .required(true)
                .value_parser(value_parser!(PathBuf)),
        )
        .arg(
            arg!(-b --blocksize <BLOCKSIZE> "block size")
                .required(false)
                .value_parser(value_parser!(u32)),
        )
        .arg(
            arg!(-l --label <LABEL> "fs label")
                .required(false)
                .value_parser(value_parser!(String)),
        )
        .arg(
            arg!(-j --journal <JOURNAL> "journal")
                .required(false)
                .value_parser(value_parser!(bool)),
        )
        .arg(
            arg!(-t --type <TYPE> "fs type")
                .required(false)
                .value_parser(value_parser!(u8)),
        )
        .get_matches();

    let path = matches.get_one::<PathBuf>("file").unwrap();
    let label = matches
        .get_one::<String>("label")
        .unwrap_or(&"ext4fs".to_string())
        .clone();
    let journal = matches.get_one::<bool>("journal").unwrap_or(&true);
    let block_size = matches.get_one::<u32>("blocksize").unwrap_or(&4096);
    let ty = matches.get_one::<u8>("type").unwrap_or(&2);
    let ty = match ty {
        2 => Ext2,
        3 => Ext3,
        4 => Ext4,
        _ => panic!("unsupported fs type"),
    };
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(path)
        .unwrap();
    let mut config = BlockDeviceConfig::default();

    let meta = file.metadata().unwrap();
    let bs: u64 = 512;
    config.block_size = bs as u32;
    config.part_size = meta.len();
    config.part_offset = 0;
    config.block_count = config.part_size / bs;
    let blk = DefaultInterface::new_device(file, config);
    let fs = FsBuilder::new()
        .ty(ty)
        .journal(*journal)
        .block_size(*block_size)
        .label(&label)
        .build(blk)
        .unwrap();
    println!("{:#x?}", fs.fs_info().unwrap());
}
