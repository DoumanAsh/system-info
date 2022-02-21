//! Network information.

pub use crate::data::network::{Ip, Address};

extern crate alloc;

use alloc::borrow::Cow;

///Iterator over socket addresses
pub struct Addresses<'a> {
    _inner: &'a Interfaces,
}

impl<'a> Addresses<'a> {
    ///Moves cursor, returning address, if it is valid IP address.
    pub fn next_addr(&mut self) -> Option<Address> {
        None
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
    _inner: &'a Interfaces,
}

impl<'a> Interface<'a> {
    #[inline(always)]
    ///Returns name of the interface, if available as utf-8 string.
    pub fn name(&'a self) -> Option<Cow<'a, str>> {
        None
    }

    #[inline(always)]
    ///Returns iterator over interface's addresses.
    pub fn addresses(&'a self) -> Addresses<'a> {
        Addresses {
            _inner: self._inner
        }
    }
}

///Iterator over [Interfaces](struct.Interfaces.html)
pub struct InterfacesIter<'a> {
    _inner: &'a Interfaces,
}

impl<'a> InterfacesIter<'a> {
    #[inline(always)]
    ///Returns current interface, if any, without moving cursor
    pub fn interface(&'a self) -> Option<Interface<'a>> {
        None
    }

    #[inline(always)]
    ///Moves cursor and returns current interface, if there is any.
    pub fn next_interface(&'a mut self) -> Option<Interface<'a>> {
        None
    }
}

impl<'a> Iterator for InterfacesIter<'a> {
    type Item = Interface<'a>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}

///Network interfaces enumerator.
pub struct Interfaces {
}

impl Interfaces {
    #[inline(always)]
    ///Creates new instance
    pub fn new() -> Option<Self> {
        Some(Self {
        })
    }

    #[inline(always)]
    ///Returns iterator over interfaces
    pub fn iter(&self) -> InterfacesIter<'_> {
        InterfacesIter {
            _inner: self
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
