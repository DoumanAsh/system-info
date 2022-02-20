//! CPU information.

use core::mem;

#[allow(non_snake_case)]
#[repr(C)]
struct SYSTEM_INFO {
    wProcessorArchitecture: u16,
    wReserved: u16,
    dwPageSize: u32,
    lpMinimumApplicationAddress: *mut u8,
    lpMaximumApplicationAddress: *mut u8,
    dwActiveProcessorMask: *mut u8,
    dwNumberOfProcessors: u32,
    dwProcessorType: u32,
    dwAllocationGranularity: u32,
    wProcessorLevel: u16,
    wProcessorRevision: u16,
}

extern "system" {
    fn GetSystemInfo(lpSystemInfo: *mut SYSTEM_INFO);
}

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
