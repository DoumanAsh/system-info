mod posix;

#[cfg(any(target_os = "linux", target_os = "android"))]
mod linux;
#[cfg(any(target_os = "linux", target_os = "android"))]
pub use linux::*;

#[cfg(not(any(target_os = "linux", target_os = "android")))]
pub use::common::cpu;
