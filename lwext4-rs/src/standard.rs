use crate::block::{BlockDevice, BlockDeviceConfig, BlockDeviceInterface};
use std::io::{Read, Seek, SeekFrom, Write};
use std::pin::Pin;

pub type SimpleBlockDevice<T> = BlockDevice<SimpleBlockDeviceInterface<T>>;

pub struct SimpleBlockDeviceInterface<T: Read + Write + Seek>(T, BlockDeviceConfig);

impl<T: Read + Write + Seek> SimpleBlockDeviceInterface<T> {
    pub fn new_device(inner: T, config: BlockDeviceConfig) -> Pin<Box<BlockDevice<Self>>> {
        BlockDevice::new(Self(inner, config))
    }
}

impl<T: Read + Write + Seek> BlockDeviceInterface for SimpleBlockDeviceInterface<T> {
    fn open(&mut self) -> crate::error::Result<BlockDeviceConfig> {
        Ok(self.1)
    }

    fn read_block(
        &mut self,
        mut buf: &mut [u8],
        block_id: u64,
        _block_count: u32,
    ) -> crate::error::Result<usize> {
        self.0.seek(SeekFrom::Start(block_id * 512)).unwrap();
        self.0.read_exact(&mut buf).unwrap();
        Ok(buf.len())
    }

    fn write_block(
        &mut self,
        buf: &[u8],
        block_id: u64,
        _block_count: u32,
    ) -> crate::error::Result<usize> {
        self.0.seek(SeekFrom::Start(block_id * 512)).unwrap();
        self.0.write_all(buf).unwrap();
        Ok(buf.len())
    }

    fn close(&mut self) -> crate::error::Result<()> {
        self.0.flush().unwrap();
        Ok(())
    }

    fn lock(&mut self) -> crate::error::Result<()> {
        Ok(())
    }

    fn unlock(&mut self) -> crate::error::Result<()> {
        Ok(())
    }
}
