//! Network information.

extern crate alloc;
extern crate std;

use windows_sys::Win32::Globalization::{CP_UTF8, WideCharToMultiByte};
use windows_sys::Win32::NetworkManagement::IpHelper::{AF_UNSPEC, AF_INET, AF_INET6, GetAdaptersAddresses, IP_ADAPTER_ADDRESSES_LH, IP_ADAPTER_UNICAST_ADDRESS_LH};
use windows_sys::Win32::NetworkManagement::IpHelper::{GAA_FLAG_SKIP_ANYCAST, GAA_FLAG_SKIP_MULTICAST, GAA_FLAG_SKIP_DNS_SERVER, GAA_FLAG_INCLUDE_PREFIX};
use windows_sys::Win32::Foundation::ERROR_BUFFER_OVERFLOW;

use core::{mem, ptr};

use std::net;
use alloc::vec::Vec;
use alloc::string::String;
use alloc::borrow::Cow;

pub use crate::data::network::Address;

impl Address {
    #[inline(always)]
    fn from(raw: &IP_ADAPTER_UNICAST_ADDRESS_LH) -> Option<Self> {
        let addr = match unsafe { raw.Address.lpSockaddr.as_ref() } {
            Some(addr) => addr,
            None => return None,
        };

        if addr.sa_family == AF_INET as u16 {
            use windows_sys::Win32::Networking::WinSock::SOCKADDR_IN;

            debug_assert_eq!(raw.Address.iSockaddrLength as usize, mem::size_of::<SOCKADDR_IN>());

            let addr: &SOCKADDR_IN = unsafe {
                mem::transmute(addr)
            };

            let addr: u32 = unsafe {
                mem::transmute(addr.sin_addr.S_un.S_addr)
            };

            // Ignore all 169.254.x.x addresses as these are not active interfaces
            if addr & 65535 == 0xfea9 {
                return None
            }

            let addr = addr.to_ne_bytes();
            let addr = net::Ipv4Addr::new(addr[0], addr[1], addr[2], addr[3]);

            Some(Self {
                ip: net::IpAddr::V4(addr),
                prefix: raw.OnLinkPrefixLength,
            })
        } else if addr.sa_family == AF_INET6 as u16 {
            use windows_sys::Win32::Networking::WinSock::SOCKADDR_IN6;

            debug_assert_eq!(raw.Address.iSockaddrLength as usize, mem::size_of::<SOCKADDR_IN6>());

            let addr: &SOCKADDR_IN6 = unsafe {
                mem::transmute(addr)
            };

            let addr: [u8; 16] = unsafe {
                mem::transmute(addr.sin6_addr.u)
            };

            // Ignore all fe80:: addresses as these are link locals
            if addr[0] == 0xfe && addr[1] == 0x80 {
                None
            } else {
                Some(Self {
                    ip: net::IpAddr::V6(net::Ipv6Addr::from(addr)),
                    prefix: raw.OnLinkPrefixLength,
                })
            }
        } else {
            None
        }
    }
}

///Iterator over socket addresses
pub struct Addresses<'a> {
    cursor: Option<&'a IP_ADAPTER_UNICAST_ADDRESS_LH>,
}

impl<'a> Addresses<'a> {
    ///Moves cursor, returning address, if it is valid IP address.
    pub fn next_addr(&mut self) -> Option<Address> {
        match self.cursor {
            Some(cursor) => unsafe {
                match cursor.Next.as_ref() {
                    Some(next) => {
                        self.cursor = Some(next);
                        match Address::from(cursor) {
                            Some(addr) => Some(addr),
                            None => self.next_addr(),
                        }
                    },
                    None => {
                        self.cursor = None;
                        Address::from(cursor)
                    },
                }
            },
            None => None,
        }
    }
}

impl<'a> Iterator for Addresses<'a> {
    type Item = Address;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.next_addr()
    }
}

///Interface
pub struct Interface<'a> {
    addrs: &'a IP_ADAPTER_ADDRESSES_LH,
}

impl<'a> Interface<'a> {
    #[inline(always)]
    ///Returns name of the interface, if available as utf-8 string.
    pub fn name(&'a self) -> Option<Cow<'a, str>> {
        if self.addrs.FriendlyName.is_null() {
            return None;
        }

        unsafe {
            //-1 to treat it as null terminating
            let req_size = WideCharToMultiByte(CP_UTF8, 0, self.addrs.FriendlyName, -1, ptr::null_mut(), 0, ptr::null(), ptr::null_mut());
            if req_size == 0 {
                None
            } else {
                let mut out = String::with_capacity(req_size as usize);
                WideCharToMultiByte(CP_UTF8, 0, self.addrs.FriendlyName, -1, out.as_mut_ptr(), req_size, ptr::null(), ptr::null_mut());
                out.as_mut_vec().set_len(req_size as usize);

                while let Some(ch) = out.pop() {
                    if ch != '\0' {
                        out.push(ch);
                        break;
                    }
                }
                Some(out.into())
            }
        }
    }

    #[inline(always)]
    ///Returns iterator over interface's addresses.
    pub fn addresses(&'a self) -> Addresses<'a> {
        Addresses {
            cursor: unsafe {
                self.addrs.FirstUnicastAddress.as_ref()
            },
        }
    }
}

///Iterator over [Interfaces](struct.Interfaces.html)
pub struct InterfacesIter<'a> {
    cursor: Option<&'a IP_ADAPTER_ADDRESSES_LH>,
}

impl<'a> InterfacesIter<'a> {
    #[inline(always)]
    ///Returns current interface, if any, without moving cursor
    pub fn interface(&'a self) -> Option<Interface<'a>> {
        self.cursor.map(|addrs| Interface {
            addrs
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
        match self.cursor {
            Some(cursor) => unsafe {
                match cursor.Next.as_ref() {
                    Some(next) => {
                        self.cursor = Some(next);
                        Some(Interface {
                            addrs: cursor,
                        })
                    },
                    None => {
                        self.cursor = None;
                        Some(Interface {
                            addrs: cursor,
                        })
                    },
                }
            },
            None => None,
        }
    }
}

///Network interfaces enumerator.
pub struct Interfaces {
    ///inner address
    buffer: Vec<u8>,
}

impl Interfaces {
    ///Creates instance.
    ///
    ///Returns `None` if syscall fails.
    pub fn new() -> Option<Self> {
        const FLAGS: u32 = GAA_FLAG_SKIP_ANYCAST |
                           GAA_FLAG_SKIP_MULTICAST |
                           GAA_FLAG_SKIP_DNS_SERVER |
                           GAA_FLAG_INCLUDE_PREFIX;

        let mut size = 0;

        let result = unsafe {
            GetAdaptersAddresses(AF_UNSPEC, FLAGS, ptr::null_mut(), ptr::null_mut(), &mut size)
        };

        if result == 0 || size == 0 {
            return Some(Self {
                buffer: Vec::new(),
            })
        }

        let mut buffer = Vec::<u8>::new();
        loop {
            buffer.reserve_exact(size as usize);
            let result = unsafe {
                GetAdaptersAddresses(AF_UNSPEC, FLAGS, ptr::null_mut(), buffer.as_mut_ptr() as _, &mut size)
            };

            match result {
                0 => {
                    unsafe {
                        buffer.set_len(size as usize);
                        break Some(Self {
                            buffer,
                        })
                    }
                },
                ERROR_BUFFER_OVERFLOW => {
                    continue;
                },
                _ => {
                    break None;
                }

            }
        }
    }

    ///Returns iterator over interfaces
    pub fn iter(&self) -> InterfacesIter<'_> {
        let cursor = match self.buffer.is_empty() {
            true => None,
            false => unsafe {
                //we trust winapi to actually write valid shit
                match (self.buffer.as_ptr() as *const IP_ADAPTER_ADDRESSES_LH).as_ref() {
                    Some(cursor) => Some(cursor),
                    None => unreach!(),
                }
            },
        };

        InterfacesIter {
            cursor,
        }
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
