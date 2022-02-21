//! Network information.

extern crate std;

use std::vec::Vec;
use std::borrow::Cow;

use core::{slice, iter};

pub use crate::data::network::Address;


#[inline(always)]
pub(crate) fn slice_c_str(input: &[u8; libc::IFNAMSIZ]) -> &[u8] {
    for idx in 0..input.len() {
        if input[idx] == 0 {
            return &input[..idx];
        }
    }

    &input[..]
}

///Iterator over socket addresses
pub struct Addresses<'a> {
    cursor: iter::Copied<slice::Iter<'a, Address>>
}

impl<'a> Addresses<'a> {
    ///Moves cursor, returning address, if it is valid IP address.
    pub fn next_addr(&mut self) -> Option<Address> {
        self.cursor.next()
    }
}

impl<'a> Iterator for Addresses<'a> {
    type Item = Address;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.next_addr()
    }
}

pub(crate) struct InterfaceData {
    pub(crate) name: [u8; libc::IFNAMSIZ],
    pub(crate) addresses: Vec<Address>
}

impl InterfaceData {
    #[inline]
    pub(crate) fn name(&self) -> &[u8] {
        slice_c_str(&self.name)
    }

    #[inline]
    pub(crate) fn push(&mut self, addr: Address) {
        self.addresses.push(addr);
    }
}

///Network interface
pub struct Interface<'a> {
    data: &'a InterfaceData
}

impl<'a> Interface<'a> {
    #[inline]
    ///Returns name of the interface, if available as utf-8 string.
    pub fn name(&'a self) -> Option<Cow<'a, str>> {
        let name = self.data.name();
        match core::str::from_utf8(name) {
            Ok(name) => Some(name.into()),
            Err(_) => None,
        }
    }

    #[inline(always)]
    ///Returns iterator over interface's addresses.
    pub fn addresses(&'a self) -> Addresses<'a> {
        Addresses {
            cursor: self.data.addresses.iter().copied()
        }
    }
}

///Iterator over [Interfaces](struct.Interfaces.html)
pub struct InterfacesIter<'a> {
    cursor: slice::Iter<'a, InterfaceData>,
}

impl<'a> InterfacesIter<'a> {
    #[inline(always)]
    ///Returns current interface, if any, without moving cursor
    pub fn interface(&'a self) -> Option<Interface<'a>> {
        self.cursor.as_slice().get(0).map(|data| Interface {
            data
        })
    }

    #[inline(always)]
    ///Moves cursor and returns current interface, if there is any.
    pub fn next_interface(&'a mut self) -> Option<Interface<'a>> {
        self.next()
    }
}

impl<'a> Iterator for InterfacesIter<'a> {
    type Item = Interface<'a>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.cursor.next().map(|data| Interface {
            data
        })
    }
}

///Network interfaces enumerator.
pub struct Interfaces {
    pub(crate) inner: Vec<InterfaceData>,
}

impl Interfaces {
    ///Returns iterator over interfaces
    pub fn iter(&self) -> InterfacesIter<'_> {
        InterfacesIter {
            cursor: self.inner.iter()
        }
    }

    #[cfg(not(any(target_os = "linux", target_os = "android")))]
    fn store_interface(&mut self, ifa_name: *const i8) -> &mut InterfaceData {
        use core::{cmp, ptr};

        let mut name = [0u8; libc::IFNAMSIZ];
        if !ifa_name.is_null() {
            unsafe {
                let len = cmp::min(name.len(), libc::strlen(ifa_name));
                ptr::copy_nonoverlapping(ifa_name, name.as_mut_ptr() as _, len)
            };
        }

        let real_name = slice_c_str(&name);

        match self.inner.binary_search_by_key(&real_name, |interface| interface.name()) {
            Ok(idx) => unsafe {
                self.inner.get_unchecked_mut(idx)
            },
            Err(idx) => {
                let interface = InterfaceData {
                    name,
                    addresses: Vec::new(),
                };
                self.inner.insert(idx, interface);

                unsafe {
                    self.inner.get_unchecked_mut(idx)
                }
            }
        }
    }

    #[cfg(not(any(target_os = "linux", target_os = "android")))]
    ///Creates new instance.
    ///
    ///In case of failure please check `std::io::Error::last_os_error()`
    pub fn new() -> Option<Self> {
        use core::mem;
        use std::net;

        struct IfAddrs(*mut libc::ifaddrs);

        impl IfAddrs {
            fn iter<'a>(&'a self) -> IfAddrsIter<'a> {
                unsafe {
                    IfAddrsIter(self.0.as_ref())
                }
            }
        }

        impl Drop for IfAddrs {
            fn drop(&mut self) {
                unsafe {
                    libc::freeifaddrs(self.0);
                }
            }
        }

        struct IfAddrsIter<'a>(Option<&'a libc::ifaddrs>);

        impl<'a> Iterator for IfAddrsIter<'a> {
            type Item = &'a libc::ifaddrs;
            fn next(&mut self) -> Option<Self::Item> {
                match self.0 {
                    Some(next) => unsafe {
                        self.0 = next.ifa_next.as_ref();
                        Some(next)
                    },
                    None => None,
                }
            }
        }

        let mut if_addrs = mem::MaybeUninit::<*mut libc::ifaddrs>::uninit();
        let if_addrs = unsafe {
            if libc::getifaddrs(if_addrs.as_mut_ptr()) != 0 {
                return None;
            }
            IfAddrs(if_addrs.assume_init())
        };

        let mut result = Interfaces {
            inner: Vec::new()
        };
        for addr in if_addrs.iter() {
            let ifa_addr = unsafe {
                addr.ifa_addr.as_ref()
            };
            //skip empty addresses
            let ifa_addr = match ifa_addr {
                Some(ifa_addr) => ifa_addr,
                None => continue,
            };

            let interface = result.store_interface(addr.ifa_name);
            if ifa_addr.sa_family == libc::AF_INET as _ {
                let ifa_addr: &libc::sockaddr_in = unsafe {
                    mem::transmute(ifa_addr)
                };

                let ip = ifa_addr.sin_addr.s_addr.to_ne_bytes();
                let ip = net::Ipv4Addr::new(ip[0], ip[1], ip[2], ip[3]);
                let net_mask = unsafe {
                    addr.ifa_netmask.as_ref()
                };
                let prefix = match net_mask {
                    //net_mask should have the same family, it is error not to
                    //which would mean getifaddrs() implementation sucks
                    Some(net_mask) if net_mask.sa_family == libc::AF_INET as _ => {
                        let net_mask: &libc::sockaddr_in = unsafe {
                            mem::transmute(net_mask)
                        };

                        net_mask.sin_addr.s_addr.count_ones() as u8
                    },
                    _ => 0
                };

                interface.push(Address {
                    ip: net::IpAddr::V4(ip),
                    prefix
                });
            } else if ifa_addr.sa_family == libc::AF_INET6 as _ {
                let ifa_addr: &libc::sockaddr_in6 = unsafe {
                    mem::transmute(ifa_addr)
                };

                let ip: [u16; 8] = unsafe {
                    mem::transmute(ifa_addr.sin6_addr.s6_addr)
                };
                let ip = net::Ipv6Addr::new(ip[0], ip[1], ip[2], ip[3], ip[4], ip[5], ip[6], ip[7]);
                let net_mask = unsafe {
                    addr.ifa_netmask.as_ref()
                };
                let prefix = match net_mask {
                    //net_mask should have the same family, it is error not to
                    //which would mean getifaddrs() implementation sucks
                    Some(net_mask) if net_mask.sa_family == libc::AF_INET6 as _ => {
                        let net_mask: &libc::sockaddr_in6 = unsafe {
                            mem::transmute(net_mask)
                        };

                        let net_mask: u128 = unsafe {
                            mem::transmute(net_mask.sin6_addr.s6_addr)
                        };
                        net_mask.count_ones() as u8
                    },
                    _ => 0
                };

                interface.push(Address {
                    ip: net::IpAddr::V6(ip),
                    prefix
                });
            }
        }

        Some(result)
    }
}

impl<'a> IntoIterator for &'a Interfaces {
    type Item = Interface<'a>;
    type IntoIter = InterfacesIter<'a>;

    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
