extern crate std;

use std::net;
use core::fmt;

#[derive(Debug, Clone, Copy)]
///Socket's address.
pub struct Address {
    ///Ip address.
    pub ip: net::IpAddr,
    ///Network address prefix.
    pub prefix: u8,
}

impl fmt::Display for Address {
    #[inline(always)]
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.ip, fmt)
    }
}

impl Address {
    #[allow(unused)]
    ///Calculates netmask from prefix.
    pub const fn net_mask(&self) -> net::IpAddr {
        match self.ip {
            net::IpAddr::V4(_) => {
                debug_assert!(self.prefix <= 32, "IPv4 prefix cannot be above 32");
                let ip = match self.prefix {
                    0 => 0u32,
                    prefix => u32::max_value().wrapping_shl(32u32.wrapping_sub(prefix as u32)),
                }.to_be_bytes();

                net::IpAddr::V4(net::Ipv4Addr::new(ip[0], ip[1], ip[2], ip[3]))
            },
            net::IpAddr::V6(_) => {
                debug_assert!(self.prefix <= 128, "IPv6 prefix cannot be above 128");
                let ip = match self.prefix {
                    0 => 0u128,
                    prefix => u128::max_value().wrapping_shl(128u32.wrapping_sub(prefix as u32)),
                }.to_be_bytes();

                let ip = [
                    u16::from_ne_bytes([ip[0], ip[1]]),
                    u16::from_ne_bytes([ip[2], ip[3]]),
                    u16::from_ne_bytes([ip[4], ip[5]]),
                    u16::from_ne_bytes([ip[6], ip[7]]),
                    u16::from_ne_bytes([ip[8], ip[9]]),
                    u16::from_ne_bytes([ip[10], ip[11]]),
                    u16::from_ne_bytes([ip[12], ip[13]]),
                    u16::from_ne_bytes([ip[14], ip[15]]),
                ];

                net::IpAddr::V6(net::Ipv6Addr::new(ip[0], ip[1], ip[2], ip[3], ip[4], ip[5], ip[6], ip[7]))
            }
        }
    }
}
