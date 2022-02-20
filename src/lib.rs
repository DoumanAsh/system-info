//! System information library

#![no_std]
#![warn(missing_docs)]
#![cfg_attr(feature = "cargo-clippy", allow(clippy::style))]

#[allow(unused)]
#[cfg(not(debug_assertions))]
macro_rules! unreach {
    () => ({
        unsafe {
            std::hint::unreachable_unchecked();
        }
    })
}

#[allow(unused)]
#[cfg(debug_assertions)]
macro_rules! unreach {
    () => ({
        unreachable!()
    })
}

mod data;

#[cfg(windows)]
mod win32;
#[cfg(windows)]
pub use win32::*;

#[cfg(unix)]
mod unix;
#[cfg(unix)]
pub use unix::*;

pub use network::Interfaces as NetworkInterfaces;
pub use os_id::{ProcessId, ThreadId, ThreadName};
