use crate::block::CName;
use crate::dir::ReadDir;
use crate::error::{errno_to_result, Result};
use lwext4_sys::ext4::*;
use std::ptr::null_mut;

#[derive(Debug)]
pub struct DirBuilder {
    recursive: bool,
}

impl DirBuilder {
    #[must_use]
    pub fn new() -> DirBuilder {
        DirBuilder { recursive: false }
    }

    pub fn recursive(&mut self, recursive: bool) -> &mut Self {
        self.recursive = recursive;
        self
    }

    pub fn create<P: AsRef<str>>(&self, path: P) -> Result<()> {
        self._create(path.as_ref())
    }

    fn _create(&self, path: &str) -> Result<()> {
        if self.recursive {
            self.create_dir_all(path)
        } else {
            self.mkdir(path)
        }
    }

    fn mkdir(&self, path: &str) -> Result<()> {
        let path = CName::new(path.to_string())?;
        unsafe { errno_to_result(ext4_dir_mk(path.as_ptr())) }
    }
    fn create_dir_all(&self, path: &str) -> Result<()> {
        self.mkdir(path)
    }
}

pub fn create_dir<P: AsRef<str>>(path: P) -> Result<()> {
    DirBuilder::new().create(path.as_ref())
}

pub fn create_dir_all<P: AsRef<str>>(path: P) -> Result<()> {
    DirBuilder::new().recursive(true).create(path.as_ref())
}

pub fn hard_link<P: AsRef<str>, Q: AsRef<str>>(original: P, link: Q) -> Result<()> {
    let original = CName::new(original.as_ref().to_string())?;
    let link = CName::new(link.as_ref().to_string())?;
    unsafe { errno_to_result(ext4_flink(original.as_ptr(), link.as_ptr())) }
}

pub fn remove_dir<P: AsRef<str>>(path: P) -> Result<()> {
    let path = CName::new(path.as_ref().to_string())?;
    unsafe { errno_to_result(ext4_dir_rm(path.as_ptr())) }
}

pub fn remove_file<P: AsRef<str>>(path: P) -> Result<()> {
    let path = CName::new(path.as_ref().to_string())?;
    unsafe { errno_to_result(ext4_fremove(path.as_ptr())) }
}

pub fn remove_dir_all<P: AsRef<str>>(_path: P) -> Result<()> {
    // for dir_en in read_dir(&path)? {
    //     let ty = dir_en.file_type();
    //     if ty.is_dir() {
    //         remove_dir_all(dir_en.path())?;
    //     } else {
    //         remove_file(dir_en.path())?;
    //     }
    // }
    Ok(())
}

pub fn rename<P: AsRef<str>, Q: AsRef<str>>(from: P, to: Q) -> Result<()> {
    // unsafe {
    //     if let (Some(from_parent), Some(from_name)) = (from.as_ref().parent(), from.as_ref().file_name().map(|x| x.to_string_lossy())) {
    //         let trim = from_name.trim_matches('\0');
    //         if let Some(entry) = read_dir(from_parent)?.find(|x| x.name() == trim) {
    //             let from_cs = CName::new(from)?;
    //             let to_cs = CName::new(to)?;
    //             errno_to_result(if entry.file_type().is_dir() {
    //                 ext4_dir_mv(from_cs.as_ptr(), to_cs.as_ptr())
    //             } else {
    //                 ext4_frename(from_cs.as_ptr(), to_cs.as_ptr())
    //             })
    //         } else {
    //             Err(Error::InvalidArgument)
    //         }
    //     } else {
    //         Err(Error::InvalidArgument)
    //     }
    // }
    let from_cs = CName::new(from.as_ref().to_string())?;
    let to_cs = CName::new(to.as_ref().to_string())?;
    unsafe { errno_to_result(ext4_frename(from_cs.as_ptr(), to_cs.as_ptr())) }
}

pub fn readdir<P: AsRef<str>>(path: P) -> Result<ReadDir> {
    let mut raw_dir = ext4_dir {
        f: ext4_file {
            mp: null_mut(),
            inode: 0,
            flags: 0,
            fsize: 0,
            fpos: 0,
        },
        de: ext4_direntry {
            inode: 0,
            entry_length: 0,
            name_length: 0,
            inode_type: 0,
            name: [0u8; 255],
        },
        next_off: 0,
    };
    let path = CName::new(path.as_ref().to_string())?;
    unsafe {
        errno_to_result(ext4_dir_open(&mut raw_dir as _, path.as_ptr()))?;
    }
    Ok(ReadDir { raw: raw_dir, path })
}
