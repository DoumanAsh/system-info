//! Memory information.

pub use crate::data::mem::SystemMemory;
use super::get_sysctl;

use core::mem;

impl SystemMemory {
    ///Fetches system information, if not available returns with all members set to 0.
    pub fn new() -> Self {
        let page_size = unsafe {
            let size = libc::sysconf(libc::_SC_PAGE_SIZE);
            if size == -1 {
                return Self {
                    total: 0,
                    avail: 0,
                }
            }

            size as u64
        };

        let total = get_sysctl([libc::CTL_HW as _, libc::HW_MEMSIZE as _]);

        let mut stats = mem::MaybeUninit::<libc::vm_statistics64>::uninit();
        let stats = unsafe {
            let mut count = libc::HOST_VM_INFO64_COUNT as _;
            let res = libc::host_statistics64(libc::mach_host_self(), libc::HOST_VM_INFO64, stats.as_mut_ptr() as *mut _, &mut count);
            if res != libc::KERN_SUCCESS {
                return Self {
                    total: 0,
                    avail: 0
                }
            }
            stats.assume_init()
        };

        Self {
            total,
            avail: total.saturating_sub((stats.active_count as u64).wrapping_add(stats.inactive_count as u64)
                                                                   .wrapping_add(stats.wire_count as u64)
                                                                   .wrapping_add(stats.speculative_count as u64)
                                                                   .wrapping_sub(stats.purgeable_count as u64)
                                                                   .saturating_mul(page_size)
            )
        }
    }
}
