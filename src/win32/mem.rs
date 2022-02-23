//! Memory information.

use core::{mem, ptr};

use windows_sys::Win32::System::SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX};

pub use crate::data::mem::SystemMemory;

impl SystemMemory {
    ///Fetches system information, if not available returns with all members set to 0.
    pub fn new() -> Self {
        let info = unsafe {
            let mut info = mem::MaybeUninit::<MEMORYSTATUSEX>::uninit();
            ptr::addr_of_mut!((*info.as_mut_ptr()).dwLength).write(mem::size_of::<MEMORYSTATUSEX>() as _);

            if GlobalMemoryStatusEx(info.as_mut_ptr()) == 0 {
                return Self {
                    total: 0,
                    avail: 0,
                }
            }
            info.assume_init()
        };

        Self {
            total: info.ullTotalPhys,
            avail: info.ullAvailPhys,
        }
    }
}
