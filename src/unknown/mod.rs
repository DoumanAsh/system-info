pub mod cpu;
pub mod network;

pub use crate::data::host::HostName;

impl HostName {
    ///Retrieves host's name.
    pub fn get() -> Option<HostName> {
        Some(HostName::new())
    }
}
