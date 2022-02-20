//! CPU information.

use core::mem;

fn count_with_affinity() -> usize {
    let mut count = 0;
    let mut set = mem::MaybeUninit::<libc::cpu_set_t>::uninit();
    let result = unsafe {
        libc::sched_getaffinity(0, mem::size_of_val(&set), set.as_mut_ptr())
    };

    if result == 0 {
        let set = unsafe {
            set.assume_init()
        };

        for idx in 0..libc::CPU_SETSIZE as usize {
            if unsafe { libc::CPU_ISSET(idx, &set) } {
                count += 1
            }
        }
    }

    count
}

///Returns number of CPU cores on system, as reported by OS.
pub fn count() -> usize {
    match count_with_affinity() {
        0 => crate::unix::posix::cpu::count(),
        count => count,
    }
}
