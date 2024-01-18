use crate::block::CName;
use crate::error::{errno_to_result, Result};
use alloc::string::ToString;
use core::ptr::null_mut;
use embedded_io::{ErrorType, Read, Seek, SeekFrom, Write};
use lwext4_sys::ext4::*;
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct FileType(pub u8);

impl FileType {
    #[must_use]
    pub fn is_dir(&self) -> bool {
        self.is(EXT4_DE_DIR as _)
    }
    #[must_use]
    pub fn is_file(&self) -> bool {
        self.is(EXT4_DE_REG_FILE as _)
    }
    #[must_use]
    pub fn is_symlink(&self) -> bool {
        self.is(EXT4_DE_SYMLINK as _)
    }
    #[must_use]
    pub fn is_block_device(&self) -> bool {
        self.is(EXT4_DE_BLKDEV as _)
    }
    #[must_use]
    pub fn is_char_device(&self) -> bool {
        self.is(EXT4_DE_CHRDEV as _)
    }
    #[must_use]
    pub fn is_fifo(&self) -> bool {
        self.is(EXT4_DE_FIFO as _)
    }
    #[must_use]
    pub fn is_socket(&self) -> bool {
        self.is(EXT4_DE_SOCK as _)
    }
    fn is(&self, ft: u8) -> bool {
        self.0 == ft
    }
}

#[derive(Clone)]
pub struct File {
    raw: ext4_file,
    path: CName,
}

impl File {
    pub fn open<P: AsRef<str>>(path: P) -> Result<Self> {
        let path = CName::new(path.as_ref().to_string())?;
        let mut raw_file = ext4_file {
            mp: null_mut(),
            inode: 0,
            flags: 0,
            fsize: 0,
            fpos: 0,
        };
        unsafe {
            errno_to_result(ext4_fopen2(&mut raw_file, path.as_ptr(), O_RDONLY as i32))?;
        }
        Ok(File {
            raw: raw_file,
            path,
        })
    }

    pub fn create<P: AsRef<str>>(path: P) -> Result<Self> {
        let path = CName::new(path.as_ref().to_string())?;
        let mut raw_file = ext4_file {
            mp: null_mut(),
            inode: 0,
            flags: 0,
            fsize: 0,
            fpos: 0,
        };
        unsafe {
            errno_to_result(ext4_fopen2(
                &mut raw_file,
                path.as_ptr(),
                (O_CREAT | O_RDWR) as i32,
            ))?;
        }
        Ok(File {
            raw: raw_file,
            path,
        })
    }
}

impl Drop for File {
    fn drop(&mut self) {
        unsafe {
            errno_to_result(ext4_fclose(&mut self.raw)).unwrap();
        }
    }
}

impl ErrorType for File {
    type Error = crate::error::Error;
}

impl Seek for File {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64> {
        let (origin, offset) = match pos {
            SeekFrom::Start(offset) => (SEEK_SET, offset as i64),
            SeekFrom::End(offset) => (SEEK_END, offset),
            SeekFrom::Current(offset) => (SEEK_CUR, offset),
        };
        unsafe {
            errno_to_result(ext4_fseek(&mut self.raw as _, offset, origin))?;
        }
        Ok(self.raw.fpos)
    }
}

impl Read for File {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        unsafe {
            let mut read = 0usize;
            let buf_size = buf.len();
            errno_to_result(ext4_fread(
                &mut self.raw as _,
                buf.as_mut_ptr() as _,
                buf_size,
                &mut read as _,
            ))?;
            Ok(read)
        }
    }
}

impl Write for File {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        unsafe {
            let mut wrote = 0usize;
            let buf_size = buf.len();
            errno_to_result(ext4_fwrite(
                &mut self.raw as _,
                buf.as_ptr() as _,
                buf_size,
                &mut wrote as _,
            ))?;
            Ok(wrote)
        }
    }

    fn flush(&mut self) -> Result<()> {
        unsafe { errno_to_result(ext4_cache_flush(self.path.as_ptr()))? }
        Ok(())
    }
}
