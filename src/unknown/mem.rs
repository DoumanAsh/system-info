//! Memory information.

pub use crate::data::mem::SystemMemory;

impl SystemMemory {
    #[inline(always)]
    ///Fetches system information, if not available returns with all members set to 0.
    pub fn new() -> Self {
        Self {
            total: 0,
            avail: 0,
        }
    }
}
