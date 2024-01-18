use crate::block::CName;
use crate::file::FileType;
use alloc::string::{String, ToString};
use core::ffi::CStr;
use core::fmt::Debug;
use core::intrinsics::transmute;
use lwext4_sys::ext4::*;

pub struct ReadDir {
    pub(super) raw: ext4_dir,
    pub(super) path: CName,
}

impl Debug for ReadDir {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ReadDir").field("path", &self.path).finish()
    }
}

impl Drop for ReadDir {
    fn drop(&mut self) {
        unsafe {
            ext4_dir_close(&mut self.raw as _);
        }
    }
}

impl ReadDir {
    /// Reset the directory stream to the beginning.
    pub fn reset(&mut self) {
        unsafe { ext4_dir_entry_rewind(&mut self.raw as _) }
    }
}

pub struct DirEntry {
    raw: ext4_direntry,
    root: CName,
}

impl DirEntry {
    pub fn name(&self) -> &str {
        unsafe {
            CStr::from_bytes_with_nul_unchecked(&self.raw.name)
                .to_str()
                .unwrap()
                .trim_matches('\0')
        }
    }

    pub fn path(&self) -> String {
        self.root.as_str().to_string() + self.name()
    }

    pub fn inode(&self) -> u32 {
        // isn't 64 bit inode supported?
        self.raw.inode
    }

    pub fn file_type(&self) -> FileType {
        FileType(self.raw.inode_type)
    }
}

impl Iterator for ReadDir {
    type Item = DirEntry;
    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            let result = ext4_dir_entry_next(&mut self.raw as _);
            if result.is_null() {
                None
            } else {
                let res = DirEntry {
                    raw: (*transmute::<_, &ext4_direntry>(result)).clone(),
                    root: self.path.clone(),
                };
                Some(res)
            }
        }
    }
}
