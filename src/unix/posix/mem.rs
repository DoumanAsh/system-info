//! Memory information.

pub use crate::data::mem::SystemMemory;

impl SystemMemory {
    ///Fetches system information, if not available returns with all members set to 0.
    pub fn new() -> Self {
        let (total_count, avail_count, size) = unsafe {
            let total_count = libc::sysconf(libc::_SC_PHYS_PAGES);
            let avail_count = libc::sysconf(libc::_SC_AVPHYS_PAGES);
            let size = libc::sysconf(libc::_SC_PAGE_SIZE);
            if total_count == -1 || avail_count == -1 || size == -1 {
                return Self {
                    total: 0,
                    avail: 0,
                }
            }

            (total_count as u64, avail_count as u64, size as u64)
        };

        Self {
            total: total_count.saturating_mul(size),
            avail: avail_count.saturating_mul(size),
        }
    }
}
