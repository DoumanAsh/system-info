use core::ptr;

use windows_sys::Win32::Globalization::{CP_UTF8, WideCharToMultiByte};
use windows_sys::Win32::System::SystemInformation::{ComputerNamePhysicalDnsHostname, GetComputerNameExW};

pub mod mem;
pub mod cpu;
pub mod network;
pub use crate::data::host::HostName;

impl HostName {
    ///Retrieves host's name.
    pub fn get() -> Option<HostName> {
        let mut buff = core::mem::MaybeUninit::<[u16; HostName::capacity()]>::uninit();
        let mut size = HostName::capacity();

        let res = unsafe {
            //retrieves size
            GetComputerNameExW(ComputerNamePhysicalDnsHostname, buff.as_mut_ptr() as *mut u16, &mut size as *mut _ as *mut _)
        };

        if res == 0 {
            return None;
        } else if size == 0 {
            return Some(HostName::new())
        }

        let mut name = [0u8; HostName::capacity()];
        unsafe {
            WideCharToMultiByte(CP_UTF8, 0,
                                buff.as_mut_ptr() as *const u16,
                                -1, name.as_mut_ptr(), name.len() as _,
                                ptr::null(), ptr::null_mut());
        }

        Some(HostName::name(name))
    }
}
