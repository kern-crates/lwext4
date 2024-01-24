use crate::block::CName;
use crate::dir::ReadDir;
use crate::error::{errno_to_result, Error, Result};
use crate::file::{raw_metadata, OpenOptions};
use crate::types::{Metadata, Permissions};
use crate::{BlockDeviceInterface, MountHandle};
use core::ffi::CStr;
use core::ptr::null_mut;
use log::info;
use lwext4_sys::ext4::*;

pub struct FileSystem<T: BlockDeviceInterface> {
    mp: MountHandle<T>,
}
impl<T: BlockDeviceInterface> Drop for FileSystem<T> {
    fn drop(&mut self) {
        info!("disable cache and stop journal");
        unsafe {
            errno_to_result(ext4_cache_write_back(self.mp.mount_point.as_ptr(), false)).unwrap();
            errno_to_result(ext4_journal_stop(self.mp.mount_point.as_ptr())).unwrap();
        }
    }
}

impl<T: BlockDeviceInterface> FileSystem<T> {
    pub fn new(mp: MountHandle<T>) -> Result<Self> {
        unsafe {
            errno_to_result(ext4_journal_start(mp.mount_point.as_ptr()))?;
            errno_to_result(ext4_cache_write_back(mp.mount_point.as_ptr(), true))?;
        }
        Ok(FileSystem { mp })
    }

    pub fn mount_handle(&self) -> &MountHandle<T> {
        &self.mp
    }

    pub fn file_builder(&self) -> OpenOptions {
        OpenOptions::new()
    }

    pub fn metadata<P: AsRef<str>>(&self, path: P) -> Result<Metadata> {
        let path = CName::new(path.as_ref().to_string())?;
        raw_metadata(&path)
    }

    pub fn readdir<P: AsRef<str>>(&self, path: P) -> Result<ReadDir> {
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

    pub fn remove_file<P: AsRef<str>>(&self, path: P) -> Result<()> {
        let path = CName::new(path.as_ref().to_string())?;
        unsafe { errno_to_result(ext4_fremove(path.as_ptr())) }
    }

    pub fn hard_link<P: AsRef<str>, Q: AsRef<str>>(&self, original: P, link: Q) -> Result<()> {
        let original = CName::new(original.as_ref().to_string())?;
        let link = CName::new(link.as_ref().to_string())?;
        unsafe { errno_to_result(ext4_flink(original.as_ptr(), link.as_ptr())) }
    }

    pub fn create_dir<P: AsRef<str>>(&self, path: P) -> Result<()> {
        let path = CName::new(path.as_ref().to_string())?;
        unsafe { errno_to_result(ext4_dir_mk(path.as_ptr())) }
    }
    pub fn create_dir_all<P: AsRef<str>>(&self, path: P) -> Result<()> {
        self.create_dir(path)
    }

    pub fn remove_dir<P: AsRef<str>>(&self, path: P) -> Result<()> {
        let path = CName::new(path.as_ref().to_string())?;
        unsafe { errno_to_result(ext4_dir_rm(path.as_ptr())) }
    }
    pub fn remove_dir_all<P: AsRef<str>>(&self, path: P) -> Result<()> {
        for dir_en in self.readdir(&path)? {
            let ty = dir_en.file_type();
            if let Ok(ty) = ty {
                if ty.is_dir() {
                    self.remove_dir_all(dir_en.path())?;
                } else {
                    self.remove_file(dir_en.path())?;
                }
            }
        }
        self.remove_dir(path)
    }

    pub fn rename<P: AsRef<str>, Q: AsRef<str>>(&self, from: P, to: Q) -> Result<()> {
        let from_cs = CName::new(from.as_ref().to_string())?;
        let to_cs = CName::new(to.as_ref().to_string())?;
        unsafe { errno_to_result(ext4_frename(from_cs.as_ptr(), to_cs.as_ptr())) }
    }

    pub fn read_link<P: AsRef<str>>(&self, path: P) -> Result<String> {
        let path = CName::new(path.as_ref().to_string())?;
        let mut buf = [0u8; 255];
        let mut read = 0usize;
        unsafe {
            errno_to_result(ext4_readlink(
                path.as_ptr(),
                buf.as_mut_ptr() as _,
                buf.len(),
                &mut read as _,
            ))?;
        }
        Ok(String::from_utf8_lossy(&buf[..read]).to_string())
    }

    pub fn set_permissions<P: AsRef<str>>(&self, path: P, perm: Permissions) -> Result<()> {
        let path = CName::new(path.as_ref().to_string())?;
        unsafe { errno_to_result(ext4_mode_set(path.as_ptr(), perm.0)) }
    }

    pub fn exists<P: AsRef<str>>(&self, path: P) -> Result<bool> {
        let path = CName::new(path.as_ref().to_string())?;
        let res = unsafe { errno_to_result(ext4_inode_exist(path.as_ptr(), 0)) };
        match res {
            Ok(_) => Ok(true),
            Err(e) => match e {
                Error::NoEntry => Ok(false),
                _ => Err(e),
            },
        }
    }

    pub fn soft_link<P: AsRef<str>, Q: AsRef<str>>(&self, original: P, link: Q) -> Result<()> {
        let original = CName::new(original.as_ref().to_string())?;
        let link = CName::new(link.as_ref().to_string())?;
        unsafe { errno_to_result(ext4_fsymlink(link.as_ptr(), original.as_ptr())) }
    }

    pub fn getxattr<P: AsRef<str>, Q: AsRef<str>>(&self, path: P, name: Q) -> Result<Vec<u8>> {
        let path = CName::new(path.as_ref().to_string())?;
        let name = CName::new(name.as_ref().to_string())?;
        let mut buf = [0u8; 255];
        let mut read = 0usize;
        unsafe {
            errno_to_result(ext4_getxattr(
                path.as_ptr(),
                name.as_ptr(),
                name.as_str().len(),
                buf.as_mut_ptr() as _,
                buf.len(),
                &mut read as _,
            ))?;
        }
        Ok(buf[..read].to_vec())
    }

    pub fn set_xattr<P: AsRef<str>, Q: AsRef<str>>(
        &self,
        path: P,
        name: Q,
        value: &[u8],
    ) -> Result<()> {
        let path = CName::new(path.as_ref().to_string())?;
        let name = CName::new(name.as_ref().to_string())?;
        unsafe {
            errno_to_result(ext4_setxattr(
                path.as_ptr(),
                name.as_ptr(),
                name.as_str().len(),
                value.as_ptr() as _,
                value.len(),
            ))?;
        }
        Ok(())
    }

    pub fn list_xattr<P: AsRef<str>>(&self, path: P) -> Result<Vec<String>> {
        let path = CName::new(path.as_ref().to_string())?;
        let mut buf = Vec::<u8>::with_capacity(255);
        let mut read = 0usize;
        loop {
            unsafe {
                errno_to_result(ext4_listxattr(
                    path.as_ptr(),
                    buf.as_mut_ptr() as _,
                    buf.len(),
                    &mut read as _,
                ))?;
            }
            if read <= buf.len() {
                break;
            }
            buf.resize(buf.len() + 255, 0);
        }
        let mut res = Vec::new();
        let mut i = 0;
        while i < read {
            let name = unsafe {
                CStr::from_bytes_with_nul_unchecked(&buf[i..])
                    .to_str()
                    .unwrap()
                    .trim_matches('\0')
            };
            res.push(name.to_string());
            i += name.len() + 1;
        }
        Ok(res)
    }

    pub fn remove_xattr<P: AsRef<str>, Q: AsRef<str>>(&self, path: P, name: Q) -> Result<()> {
        let path = CName::new(path.as_ref().to_string())?;
        let name = CName::new(name.as_ref().to_string())?;
        unsafe {
            errno_to_result(ext4_removexattr(
                path.as_ptr(),
                name.as_ptr(),
                name.as_str().len(),
            ))?;
        }
        Ok(())
    }
}
