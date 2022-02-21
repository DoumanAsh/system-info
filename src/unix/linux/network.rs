//! Network information.

extern crate alloc;

use alloc::vec::Vec;

use core::mem;

pub use crate::data::network::{Ip, Address};
pub use crate::unix::posix::network::Interfaces;
use crate::unix::posix::network::{slice_c_str, InterfaceData};

const ALIGN_SIZE: usize = 4;
const NETLINK_HEADER_SIZE: usize = mem::size_of::<libc::nlmsghdr>();
const NETLINK_ADDR_REQ_SIZE: usize = mem::size_of::<NetlinkAddrReq>();

#[repr(C)]
#[repr(align(4))]
struct RtaAttr {
    rta_len: u16,
    rta_type: u16,
}

impl RtaAttr {
    #[inline(always)]
    fn is_ok(&self, payload_len: u32) -> bool {
        payload_len >= mem::size_of::<Self>() as u32 &&
        self.rta_len as usize >= mem::size_of::<RtaAttr>() &&
        self.rta_len as u32 <= payload_len
    }
}

#[repr(C)]
struct IfAddrMsg {
    ifa_family: u8,
    ifa_prefixlen: u8,
    ifa_flags: u8,
    ifa_scope: u8,
    ifa_index: u32,
}

#[repr(C)]
#[repr(align(4))]
struct NetlinkAddrReq {
    header: libc::nlmsghdr,
    msg: IfAddrMsg,
}

unsafe fn extract_rta_data<T: Copy>(rta_attr: &RtaAttr) -> T {
    let mut out = mem::MaybeUninit::<T>::zeroed();

    let rta_data = (rta_attr as *const _ as *const u8).add(mem::size_of_val(rta_attr)) as *const u8;
    let rta_len = (rta_attr.rta_len as usize) - mem::size_of_val(rta_attr);

    (out.as_mut_ptr() as *mut u8).copy_from_nonoverlapping(rta_data, rta_len as usize);
    out.assume_init()
}

struct Socket {
    fd: libc::c_int,
    addr: libc::sockaddr_nl,
}

impl Socket {
    fn new() -> Option<Self> {
        let mut addr = unsafe {
            mem::MaybeUninit::<libc::sockaddr_nl>::zeroed().assume_init()
        };
        addr.nl_family = libc::AF_NETLINK as _;

        socket().map(|fd| Self {
            fd,
            addr
        })
    }

    fn send(&self, mut msg: NetlinkAddrReq) -> bool {
        let mut msg = libc::iovec {
            iov_base: &mut msg as *mut _ as *mut _,
            iov_len: msg.header.nlmsg_len as _,
        };
        let mut req = unsafe {
            mem::MaybeUninit::<libc::msghdr>::zeroed().assume_init()
        };
        req.msg_name = &self.addr as *const _ as *mut _;
        req.msg_namelen = mem::size_of_val(&self.addr) as _;
        req.msg_iov = &mut msg as *mut _ as *mut _;
        req.msg_iovlen = 1;

        let res = unsafe {
            libc::sendmsg(self.fd, &mut req as *mut _, 0)
        };
        res >= 0
    }

    fn recv(&self, buffer: &mut [u8]) -> Option<usize> {
        let mut msg = libc::iovec {
            iov_base: buffer.as_mut_ptr() as *mut _,
            iov_len: buffer.len(),
        };
        let mut req = unsafe {
            mem::MaybeUninit::<libc::msghdr>::zeroed().assume_init()
        };
        req.msg_name = &self.addr as *const _ as *mut _;
        req.msg_namelen = mem::size_of_val(&self.addr) as _;
        req.msg_iov = &mut msg as *mut _ as *mut _;
        req.msg_iovlen = 1;

        let res = unsafe {
            libc::recvmsg(self.fd, &mut req as *mut _, 0)
        };

        if res < 0  {
            None
        } else {
            Some(res as usize)
        }

    }
}

impl Drop for Socket {
    #[inline(always)]
    fn drop(&mut self) {
        unsafe {
            libc::close(self.fd);
        }
    }
}

fn socket() -> Option<libc::c_int> {
    let fd = unsafe {
        libc::socket(libc::AF_NETLINK, libc::SOCK_DGRAM | libc::SOCK_CLOEXEC, libc::NETLINK_ROUTE)
    };

    if fd == -1 {
        return None;
    }

    Some(fd)
}

impl Interfaces {
    #[inline(always)]
    //It can fail if interface_index is invalid
    fn store_interface(&mut self, interface_index: u32) -> Option<&mut InterfaceData> {
        let mut name = [0u8; libc::IFNAMSIZ];
        let result = unsafe {
            libc::if_indextoname(interface_index, name.as_mut_ptr() as _)
        };

        if result.is_null() {
            None
        } else {
            let real_name = slice_c_str(&name);
            match self.inner.binary_search_by_key(&real_name, |interface| interface.name()) {
                Ok(idx) => Some(unsafe {
                    self.inner.get_unchecked_mut(idx)
                }),
                Err(idx) => {
                    let interface = InterfaceData {
                        name,
                        addresses: Vec::new(),
                    };
                    self.inner.insert(idx, interface);

                    Some(unsafe {
                        self.inner.get_unchecked_mut(idx)
                    })
                }
            }
        }
    }

    ///Creates new instance.
    ///
    ///In case of failure please check `std::io::Error::last_os_error()`
    pub fn new() -> Option<Self> {
        let netlink = Socket::new()?;
        let mut req = unsafe {
            mem::MaybeUninit::<NetlinkAddrReq>::zeroed().assume_init()
        };
        req.header.nlmsg_flags = (libc::NLM_F_REQUEST | libc::NLM_F_DUMP) as u16;
        req.header.nlmsg_type = 22; //RTM_GETADDR
        req.header.nlmsg_len = NETLINK_ADDR_REQ_SIZE as u32;
        req.msg.ifa_family = libc::AF_UNSPEC as _; //All IPs
        req.msg.ifa_index = 0; //All interfaces

        if !netlink.send(req) {
            return None;
        }

        let mut result = Interfaces {
            inner: Vec::new()
        };
        let mut buf = [0u8; 65536];
        while let Some(mut size) = netlink.recv(&mut buf) {
            let mut cursor_ptr = buf.as_ptr();
            let mut cursor = unsafe {
                &*(cursor_ptr as *const NetlinkAddrReq)
            };

            const DONE: u16 = libc::NLMSG_DONE as u16;
            const ERROR: u16 = libc::NLMSG_ERROR as u16;
            const NEW_ADDR: u16 = 20; //RTM_NEWADDR

            while size >= NETLINK_HEADER_SIZE && cursor.header.nlmsg_len >= NETLINK_HEADER_SIZE as u32 && cursor.header.nlmsg_len <= size as u32 {
                match cursor.header.nlmsg_type {
                    DONE => return Some(result),
                    ERROR => return None,
                    NEW_ADDR => unsafe {
                        let if_req = &cursor.msg;
                        let mut data_len = cursor.header.nlmsg_len - mem::size_of_val(cursor) as u32;
                        let rta_attr = cursor_ptr.add(mem::size_of_val(cursor)) as *const RtaAttr;
                        let mut rta_attr = &*rta_attr;

                        while rta_attr.is_ok(data_len) {
                            if rta_attr.rta_type == 2  {
                                //IFA_LOCAL
                                //RTM_GETADDR only responds with ipv4
                                if if_req.ifa_family == libc::AF_INET as u8 {
                                    let interface = result.store_interface(if_req.ifa_index)?;

                                    let ip = extract_rta_data::<[u8; mem::size_of::<u32>()]>(rta_attr);
                                    let ip = Ip::V4(ip);

                                    interface.push(Address {
                                        ip,
                                        prefix: if_req.ifa_prefixlen,
                                    });
                                }
                            } else if rta_attr.rta_type == 1  {
                                //IFA_ADDRESS
                                //RTM_GETADDR responds with ipv6
                                if if_req.ifa_family == libc::AF_INET6 as u8 {
                                    let interface = result.store_interface(if_req.ifa_index)?;
                                    let ip = extract_rta_data::<[u16; 8]>(rta_attr);
                                    let ip = Ip::V6(ip);

                                    interface.push(Address {
                                        ip,
                                        prefix: if_req.ifa_prefixlen,
                                    });
                                }
                            }

                            //go to next RTA
                            let rta_size = (rta_attr.rta_len as usize + ALIGN_SIZE - 1) & !(ALIGN_SIZE - 1); //aligned
                            data_len -= rta_size as u32;
                            rta_attr = &*((rta_attr as *const _ as *const u8).add(rta_size) as *const RtaAttr);
                        }
                    },
                    //we don't care about anything else
                    _ => (),
                }

                //Go to next message
                let msg_size = (cursor.header.nlmsg_len as usize + ALIGN_SIZE - 1) & !(ALIGN_SIZE - 1); //aligned
                size -= msg_size;
                unsafe {
                    cursor_ptr = cursor_ptr.add(msg_size);
                    cursor = &*(cursor_ptr as *const NetlinkAddrReq);
                }
            }
        }

        //Failed to read from socket
        None
    }
}
