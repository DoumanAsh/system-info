#[cfg(feature = "std")]
extern crate std;

use core::fmt;

///IP address
#[derive(Debug, Clone, Copy)]
pub enum Ip {
    ///IP version 4
    V4([u8; 4]),
    ///IP version 6
    V6([u16; 8]),
}

#[inline(always)]
const fn is_v4_unspecified(ip: [u8; 4]) -> bool {
    u32::from_ne_bytes(ip) == 0
}

#[inline(always)]
const fn is_v4_loopback(ip: [u8; 4]) -> bool {
    ip[0] == 127
}

#[inline(always)]
const fn is_v6_unspecified(ip: [u16; 8]) -> bool {
    ip[0] == 0 && ip[1] == 0 && ip[2] == 0 && ip[3] == 0 && ip[4] == 0 && ip[5] == 0 && ip[6] == 0 && ip[7] == 0
}

#[inline(always)]
const fn is_v6_loopback(ip: [u16; 8]) -> bool {
    ip[0] == 0 && ip[1] == 0 && ip[2] == 0 && ip[3] == 0 && ip[4] == 0 && ip[5] == 0 && ip[6] == 0 && ip[7] == 1
}

impl Ip {
    #[cfg(feature = "std")]
    ///Converts to `std` `IpAddr`
    ///
    ///Requires `std` feature.
    pub const fn to_std(&self) -> std::net::IpAddr {
        match self {
            Ip::V4(addr) => std::net::IpAddr::V4(std::net::Ipv4Addr::new(addr[0], addr[1], addr[2], addr[3])),
            Ip::V6(addr) => std::net::IpAddr::V6(std::net::Ipv6Addr::new(addr[0], addr[1], addr[2], addr[3], addr[4], addr[5], addr[6], addr[7]))
        }
    }

    ///Returns whether it is `unspecified` IP (namely zero IP).
    pub const fn is_unspecified(&self) -> bool {
        match self {
            Ip::V4(addr) => is_v4_unspecified(*addr),
            Ip::V6(addr) => is_v6_unspecified(*addr),
        }
    }

    ///Returns whether it is `loopback` IP.
    pub const fn is_loopback(&self) -> bool {
        match self {
            Ip::V4(addr) => is_v4_loopback(*addr),
            Ip::V6(addr) => is_v6_loopback(*addr),
        }
    }
}

impl fmt::Display for Ip {
    #[inline(always)]
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Ip::V4(addr) => {
                fmt.write_fmt(format_args!("{}.{}.{}.{}", addr[0], addr[1], addr[2], addr[3]))
            },
            Ip::V6(addr) => {
                if is_v6_unspecified(*addr) {
                    fmt.write_str("::")
                } else if is_v6_loopback(*addr) {
                    fmt.write_str("::1")
                } else {
                    fmt.write_fmt(format_args!("{:x}:{:x}:{:x}:{:x}:{:x}:{:x}:{:x}:{:x}", addr[0], addr[1], addr[2], addr[3], addr[4], addr[5], addr[6], addr[7]))
                }
            },
        }
    }
}

#[derive(Debug, Clone, Copy)]
///Socket's address.
pub struct Address {
    ///Ip address.
    pub ip: Ip,
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
    pub const fn net_mask(&self) -> Ip {
        match self.ip {
            Ip::V4(_) => {
                debug_assert!(self.prefix <= 32, "IPv4 prefix cannot be above 32");
                let ip = match self.prefix {
                    0 => 0u32,
                    prefix => u32::max_value().wrapping_shl(32u32.wrapping_sub(prefix as u32)),
                }.to_be_bytes();

                Ip::V4(ip)
            },
            Ip::V6(_) => {
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

                Ip::V6(ip)
            }
        }
    }
}
