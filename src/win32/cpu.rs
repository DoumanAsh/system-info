//! CPU information.

use core::mem;

use windows_sys::Win32::System::SystemInformation::{SYSTEM_INFO, GetSystemInfo};

fn system_info() -> SYSTEM_INFO {
    let mut info = mem::MaybeUninit::<SYSTEM_INFO>::uninit();
    unsafe {
        GetSystemInfo(info.as_mut_ptr());
        info.assume_init()
    }

}

///Returns number of CPU cores on system, as reported by OS.
pub fn count() -> usize {
    system_info().dwNumberOfProcessors as usize
}
