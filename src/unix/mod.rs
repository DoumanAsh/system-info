mod posix;
#[cfg(not(any(target_os = "linux", target_os = "android", target_os = "macos", target_os = "ios")))]
pub use posix::*;

#[cfg(any(target_os = "macos", target_os = "ios"))]
mod apple;
#[cfg(any(target_os = "macos", target_os = "ios"))]
pub use apple::*;

#[cfg(any(target_os = "linux", target_os = "android"))]
mod linux;
#[cfg(any(target_os = "linux", target_os = "android"))]
pub use linux::*;

pub use crate::data::host::HostName;

impl HostName {
    ///Retrieves host's name.
    pub fn get() -> Option<HostName> {
        let mut name = [0u8; HostName::capacity()];
        let res = unsafe {
            libc::gethostname(name.as_mut_ptr() as _, name.len())
        };

        if res == 0 {
            Some(HostName::name(name))
        } else {
            None
        }
    }
}
