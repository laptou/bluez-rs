use std::io::{Read, Write};
use std::mem::MaybeUninit;
use std::os::unix::net::{UnixListener, UnixStream};
use std::os::unix::prelude::IntoRawFd;

use libc::{self, c_int};
use std::os::unix::io::{AsRawFd, FromRawFd, RawFd};

use crate::management::client::AddressType;
use crate::util::check_error;
use crate::{socket::*, Address};

#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct SockAddrL2 {
    pub l2_family: i16,
    pub l2_psm: u16,
    pub l2_bdaddr: Address,
    pub l2_cid: u16,
    pub l2_bdaddr_type: AddressType,
}

impl Default for SockAddrL2 {
    fn default() -> Self {
        Self {
            l2_family: Default::default(),
            l2_psm: Default::default(),
            l2_bdaddr: Default::default(),
            l2_cid: Default::default(),
            l2_bdaddr_type: AddressType::BREDR,
        }
    }
}

#[repr(C)]
struct L2capOptions {
    omtu: u16,
    imtu: u16,
    flush_to: u16,
    mode: u8,
}

pub const SOL_L2CAP: c_int = 6;

pub struct L2capListener {
    inner: UnixListener,
}

impl L2capListener {
    pub fn bind(addr: Address, addr_type: AddressType, port: u16) -> Result<Self, std::io::Error> {
        let fd: RawFd = check_error(unsafe {
            libc::socket(
                libc::AF_BLUETOOTH,
                libc::SOCK_CLOEXEC | libc::SOCK_SEQPACKET,
                BtProto::L2CAP as libc::c_int,
            )
        })?;

        let addr = SockAddrL2 {
            l2_family: libc::AF_BLUETOOTH as i16,
            l2_bdaddr: addr,
            l2_bdaddr_type: addr_type,
            l2_psm: port,
            ..Default::default()
        };

        if let Err(err) = check_error(unsafe {
            libc::bind(
                fd,
                &addr as *const SockAddrL2 as *const libc::sockaddr,
                std::mem::size_of::<SockAddrL2>() as u32,
            )
        }) {
            unsafe {
                libc::close(fd);
            }

            return Err(err);
        }

        if let Err(err) = check_error(unsafe {
            libc::listen(
                fd, 128
            )
        }) {
            unsafe {
                libc::close(fd);
            }

            return Err(err);
        }   

        let listener = unsafe { UnixListener::from_raw_fd(fd) };

        Ok(L2capListener { inner: listener })
    }

    pub fn accept(&self) -> Result<(L2capStream, (Address, u16)), std::io::Error> {
        let mut addr: SockAddrL2 = Default::default();
        let mut addr_size = std::mem::size_of::<SockAddrL2>() as u32;

        let fd = check_error(unsafe {
            libc::accept(
                self.inner.as_raw_fd(),
                &mut addr as *mut _ as *mut _,
                &mut addr_size,
            )
        })?;

        let addr = (addr.l2_bdaddr, addr.l2_psm);
        let sock = unsafe { L2capStream::from_raw_fd(fd) };

        Ok((sock, addr))
    }

    pub fn local_addr(&self) -> Result<(Address, u16), std::io::Error> {
        let mut addr: SockAddrL2 = Default::default();
        let mut addr_size = std::mem::size_of::<SockAddrL2>() as u32;

        check_error(unsafe {
            libc::getsockname(
                self.inner.as_raw_fd(),
                &mut addr as *mut _ as *mut _,
                &mut addr_size,
            )
        })?;

        Ok((addr.l2_bdaddr, addr.l2_psm))
    }

    pub fn incoming(&self) -> Incoming<'_> {
        Incoming { listener: self }
    }
}

impl AsRawFd for L2capListener {
    fn as_raw_fd(&self) -> RawFd {
        self.inner.as_raw_fd()
    }
}

impl FromRawFd for L2capListener {
    unsafe fn from_raw_fd(fd: RawFd) -> L2capListener {
        let listener = UnixListener::from_raw_fd(fd);
        L2capListener { inner: listener }
    }
}

impl IntoRawFd for L2capListener {
    fn into_raw_fd(self) -> RawFd {
        self.inner.into_raw_fd()
    }
}

impl<'a> IntoIterator for &'a L2capListener {
    type Item = std::io::Result<L2capStream>;
    type IntoIter = Incoming<'a>;

    fn into_iter(self) -> Incoming<'a> {
        self.incoming()
    }
}

pub struct Incoming<'a> {
    listener: &'a L2capListener,
}

impl<'a> Iterator for Incoming<'a> {
    type Item = std::io::Result<L2capStream>;

    fn next(&mut self) -> Option<std::io::Result<L2capStream>> {
        Some(self.listener.accept().map(|s| s.0))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (usize::MAX, None)
    }
}

#[derive(Debug)]
pub struct L2capStream {
    inner: UnixStream,
}

impl L2capStream {
    pub fn connect(
        addr: Address,
        addr_type: AddressType,
        port: u16,
    ) -> Result<Self, smol::io::Error> {
        let fd: RawFd = check_error(unsafe {
            libc::socket(
                libc::AF_BLUETOOTH,
                libc::SOCK_CLOEXEC | libc::SOCK_SEQPACKET,
                BtProto::L2CAP as libc::c_int,
            )
        })?;

        let addr = SockAddrL2 {
            l2_family: libc::AF_BLUETOOTH as i16,
            l2_bdaddr: addr,
            l2_bdaddr_type: addr_type,
            l2_psm: port,
            ..Default::default()
        };

        if let Err(err) = check_error(unsafe {
            libc::connect(
                fd,
                &addr as *const SockAddrL2 as *const libc::sockaddr,
                std::mem::size_of::<SockAddrL2>() as u32,
            )
        }) {
            unsafe {
                libc::close(fd);
            }

            return Err(err);
        }

        let stream = unsafe { std::os::unix::net::UnixStream::from_raw_fd(fd) };

        Ok(L2capStream { inner: stream })
    }

    pub fn set_mtu(&mut self, mtu: u16) -> std::io::Result<()> {
        let mut options = std::mem::MaybeUninit::<L2capOptions>::uninit();
        let mut len = std::mem::size_of::<L2capOptions>() as u32;

        check_error(unsafe {
            libc::getsockopt(
                self.inner.as_raw_fd(),
                SOL_L2CAP,
                0x01, /* L2CAP_OPTIONS */
                &mut options as *mut MaybeUninit<L2capOptions> as *mut _,
                &mut len,
            )
        })?;

        let mut options = unsafe { options.assume_init() };

        options.omtu = mtu;
        options.imtu = mtu;

        check_error(unsafe {
            libc::setsockopt(
                self.inner.as_raw_fd(),
                SOL_L2CAP,
                0x01, /* L2CAP_OPTIONS */
                &options as *const L2capOptions as *const libc::c_void,
                len,
            )
        })?;

        Ok(())
    }

    pub fn set_nonblocking(&self, nonblocking: bool) -> std::io::Result<()> {
        self.inner.set_nonblocking(nonblocking)
    }

    pub fn local_addr(&self) -> Result<(Address, u16), std::io::Error> {
        let mut addr: SockAddrL2 = Default::default();
        let mut addr_size = std::mem::size_of::<SockAddrL2>() as u32;

        check_error(unsafe {
            libc::getsockname(
                self.inner.as_raw_fd(),
                &mut addr as *mut _ as *mut _,
                &mut addr_size,
            )
        })?;

        Ok((addr.l2_bdaddr, addr.l2_psm))
    }

    pub fn peer_addr(&self) -> Result<(Address, u16), std::io::Error> {
        let mut addr: SockAddrL2 = Default::default();
        let mut addr_size = std::mem::size_of::<SockAddrL2>() as u32;

        check_error(unsafe {
            libc::getpeername(
                self.inner.as_raw_fd(),
                &mut addr as *mut _ as *mut _,
                &mut addr_size,
            )
        })?;

        Ok((addr.l2_bdaddr, addr.l2_psm))
    }
}

impl AsRawFd for L2capStream {
    fn as_raw_fd(&self) -> RawFd {
        self.inner.as_raw_fd()
    }
}

impl FromRawFd for L2capStream {
    unsafe fn from_raw_fd(fd: RawFd) -> Self {
        let stream = UnixStream::from_raw_fd(fd);
        L2capStream { inner: stream }
    }
}

impl IntoRawFd for L2capStream {
    fn into_raw_fd(self) -> RawFd {
        self.inner.into_raw_fd()
    }
}

impl Read for L2capStream {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.inner.read(buf)
    }
}

impl<'a> Read for &'a L2capStream {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        (&(*self).inner).read(buf)
    }
}

impl Write for L2capStream {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.inner.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
    }
}

impl<'a> Write for &'a L2capStream {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        (&(*self).inner).write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        (&(*self).inner).flush()
    }
}
