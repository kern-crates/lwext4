#![feature(try_blocks)]
#![feature(error_in_core)]
#![cfg_attr(not(feature = "std"), no_std)]

mod block;
mod dir;
mod error;
mod file;

#[cfg(feature = "std")]
mod standard;

extern crate alloc;

#[cfg(feature = "std")]
pub use standard::*;

pub use block::*;
pub mod fs;
