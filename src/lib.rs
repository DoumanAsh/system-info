//! System information library
//!
//!## Requirements
//!
//!- `alloc` - Requires heap allocations to store network data on unix & windows systems.
//!
//!## Features
//!
//!- `std` - Enables std's types support;

#![no_std]
#![warn(missing_docs)]
#![cfg_attr(feature = "cargo-clippy", allow(clippy::style))]
#![cfg_attr(feature = "cargo-clippy", allow(clippy::needless_lifetimes))]

#[allow(unused)]
#[cfg(not(debug_assertions))]
macro_rules! unreach {
    () => ({
        #[allow(unused_unsafe)]
        unsafe {
            core::hint::unreachable_unchecked();
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

#[cfg(not(any(unix, windows)))]
mod unknown;
#[cfg(not(any(unix, windows)))]
pub use unknown::*;

#[cfg(windows)]
mod win32;
#[cfg(windows)]
pub use win32::*;

#[cfg(unix)]
mod unix;
#[cfg(unix)]
pub use unix::*;

#[cfg(any(unix, windows))]
pub use network::Interfaces as NetworkInterfaces;
pub use os_id::{ProcessId, ThreadId, ThreadName};
