pub use super::posix::network;
pub use super::posix::cpu;
pub mod mem;

use core::ptr;

pub(crate) fn get_sysctl<T: Copy>(mut name: [i32; 2]) -> T {
    let mut result = core::mem::MaybeUninit::<T>::zeroed();
    unsafe {
        let mut result_len = core::mem::size_of::<T>() as libc::size_t;
        libc::sysctl(name.as_mut_ptr(), name.len() as _, result.as_mut_ptr() as _, &mut result_len as *mut _, ptr::null_mut(), 0);
        result.assume_init()
    }
}
