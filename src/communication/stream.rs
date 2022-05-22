use std::io::{Read, Write};
use std::mem::MaybeUninit;
use std::os::unix::net::UnixStream;
use std::os::unix::prelude::IntoRawFd;

use libc;
use std::os::unix::io::{AsRawFd, FromRawFd, RawFd};

use crate::management::client::AddressType;
use crate::util::check_error;
use crate::{socket::*, Address};

union SockAddr {
    l2: bluetooth_sys::sockaddr_l2,
    rc: bluetooth_sys::sockaddr_rc,
}

pub struct BluetoothListener {
    inner: RawFd,
    proto: BtProto,
}

impl BluetoothListener {
    pub fn bind(
        proto: BtProto,
        addr: Address,
        addr_type: AddressType,
        port: u16,
    ) -> Result<Self, std::io::Error> {
        let flags = match proto {
            BtProto::L2CAP => libc::SOCK_SEQPACKET,
            BtProto::RFCOMM => libc::SOCK_STREAM,
            other => panic!(
                "bluetooth protocol {:?} cannot be used with BluetoothListener",
                other
            ),
        };

        let fd: RawFd = check_error(unsafe {
            libc::socket(
                libc::AF_BLUETOOTH,
                libc::SOCK_CLOEXEC | flags,
                proto as libc::c_int,
            )
        })?;

        let (addr, addr_len) = match proto {
            BtProto::L2CAP => (
                SockAddr {
                    l2: bluetooth_sys::sockaddr_l2 {
                        l2_family: libc::AF_BLUETOOTH as u16,
                        l2_bdaddr: addr.into(),
                        l2_bdaddr_type: addr_type as u8,
                        l2_psm: port,
                        l2_cid: 0,
                    },
                },
                std::mem::size_of::<bluetooth_sys::sockaddr_l2>(),
            ),
            BtProto::RFCOMM => (
                SockAddr {
                    rc: bluetooth_sys::sockaddr_rc {
                        rc_family: libc::AF_BLUETOOTH as u16,
                        rc_bdaddr: addr.into(),
                        rc_channel: port as u8,
                    },
                },
                std::mem::size_of::<bluetooth_sys::sockaddr_rc>(),
            ),
            _ => unreachable!(),
        };

        if let Err(err) = check_error(unsafe {
            libc::bind(
                fd,
                &addr as *const SockAddr as *const libc::sockaddr,
                addr_len as u32,
            )
        }) {
            unsafe {
                libc::close(fd);
            }

            return Err(err);
        }

        if let Err(err) = check_error(unsafe { libc::listen(fd, 128) }) {
            unsafe {
                libc::close(fd);
            }

            return Err(err);
        }

        Ok(BluetoothListener { inner: fd, proto })
    }

    pub fn accept(&self) -> Result<(BluetoothStream, (Address, u16)), std::io::Error> {
        let mut addr: SockAddr = unsafe { std::mem::zeroed() };
        let mut addr_len = match self.proto {
            BtProto::L2CAP => std::mem::size_of::<bluetooth_sys::sockaddr_l2>(),
            BtProto::RFCOMM => std::mem::size_of::<bluetooth_sys::sockaddr_rc>(),
            _ => unreachable!(),
        } as u32;

        let fd = check_error(unsafe {
            libc::accept(self.inner, &mut addr as *mut _ as *mut _, &mut addr_len)
        })?;

        let addr = match self.proto {
            BtProto::L2CAP => unsafe { (addr.l2.l2_bdaddr.into(), addr.l2.l2_psm) },
            BtProto::RFCOMM => unsafe { (addr.rc.rc_bdaddr.into(), addr.rc.rc_channel as u16) },
            _ => unreachable!(),
        };

        let sock = unsafe {
            BluetoothStream {
                inner: UnixStream::from_raw_fd(fd),
                proto: self.proto,
            }
        };

        Ok((sock, addr))
    }

    pub fn local_addr(&self) -> Result<(Address, u16), std::io::Error> {
        let mut addr: SockAddr = unsafe { std::mem::zeroed() };
        let mut addr_len = match self.proto {
            BtProto::L2CAP => std::mem::size_of::<bluetooth_sys::sockaddr_l2>(),
            BtProto::RFCOMM => std::mem::size_of::<bluetooth_sys::sockaddr_rc>(),
            _ => unreachable!(),
        } as u32;

        check_error(unsafe {
            libc::getsockname(
                self.inner.as_raw_fd(),
                &mut addr as *mut _ as *mut _,
                &mut addr_len,
            )
        })?;

        let addr = match self.proto {
            BtProto::L2CAP => unsafe { (addr.l2.l2_bdaddr.into(), addr.l2.l2_psm) },
            BtProto::RFCOMM => unsafe { (addr.rc.rc_bdaddr.into(), addr.rc.rc_channel as u16) },
            _ => unreachable!(),
        };

        Ok(addr)
    }

    pub fn incoming(&self) -> Incoming<'_> {
        Incoming { listener: self }
    }
}

impl AsRawFd for BluetoothListener {
    fn as_raw_fd(&self) -> RawFd {
        self.inner
    }
}

impl IntoRawFd for BluetoothListener {
    fn into_raw_fd(self) -> RawFd {
        self.inner
    }
}

impl<'a> IntoIterator for &'a BluetoothListener {
    type Item = std::io::Result<BluetoothStream>;
    type IntoIter = Incoming<'a>;

    fn into_iter(self) -> Incoming<'a> {
        self.incoming()
    }
}

pub struct Incoming<'a> {
    listener: &'a BluetoothListener,
}

impl<'a> Iterator for Incoming<'a> {
    type Item = std::io::Result<BluetoothStream>;

    fn next(&mut self) -> Option<std::io::Result<BluetoothStream>> {
        Some(self.listener.accept().map(|s| s.0))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (usize::MAX, None)
    }
}

#[derive(Debug)]
pub struct BluetoothStream {
    inner: UnixStream,
    proto: BtProto,
}

impl BluetoothStream {
    pub fn connect(
        proto: BtProto,
        addr: Address,
        addr_type: AddressType,
        port: u16,
    ) -> Result<Self, smol::io::Error> {
        let flags = match proto {
            BtProto::L2CAP => libc::SOCK_SEQPACKET,
            BtProto::RFCOMM => libc::SOCK_STREAM,
            other => panic!(
                "bluetooth protocol {:?} cannot be used with BluetoothStream",
                other
            ),
        };

        let fd: RawFd = check_error(unsafe {
            libc::socket(
                libc::AF_BLUETOOTH,
                libc::SOCK_CLOEXEC | flags,
                proto as libc::c_int,
            )
        })?;

        let (addr, addr_len) = match proto {
            BtProto::L2CAP => (
                SockAddr {
                    l2: bluetooth_sys::sockaddr_l2 {
                        l2_family: libc::AF_BLUETOOTH as u16,
                        l2_bdaddr: addr.into(),
                        l2_bdaddr_type: addr_type as u8,
                        l2_psm: port,
                        l2_cid: 0,
                    },
                },
                std::mem::size_of::<bluetooth_sys::sockaddr_l2>(),
            ),
            BtProto::RFCOMM => (
                SockAddr {
                    rc: bluetooth_sys::sockaddr_rc {
                        rc_family: libc::AF_BLUETOOTH as u16,
                        rc_bdaddr: addr.into(),
                        rc_channel: port as u8,
                    },
                },
                std::mem::size_of::<bluetooth_sys::sockaddr_rc>(),
            ),
            _ => unreachable!(),
        };

        if let Err(err) = check_error(unsafe {
            libc::connect(
                fd,
                &addr as *const SockAddr as *const libc::sockaddr,
                addr_len as u32,
            )
        }) {
            unsafe {
                libc::close(fd);
            }

            return Err(err);
        }

        let stream = unsafe { UnixStream::from_raw_fd(fd) };

        Ok(BluetoothStream {
            inner: stream,
            proto,
        })
    }

    pub(crate) unsafe fn from_raw_fd(fd: RawFd, proto: BtProto) -> Self {
        BluetoothStream {
            inner: UnixStream::from_raw_fd(fd),
            proto,
        }
    }

    pub fn set_mtu(&mut self, mtu: u16) -> std::io::Result<()> {
        let mut options = std::mem::MaybeUninit::<bluetooth_sys::l2cap_options>::uninit();
        let mut len = std::mem::size_of::<bluetooth_sys::l2cap_options>() as u32;

        check_error(unsafe {
            libc::getsockopt(
                self.inner.as_raw_fd(),
                bluetooth_sys::SOL_L2CAP as i32,
                0x01, /* L2CAP_OPTIONS */
                &mut options as *mut MaybeUninit<bluetooth_sys::l2cap_options> as *mut _,
                &mut len,
            )
        })?;

        let mut options = unsafe { options.assume_init() };

        options.omtu = mtu;
        options.imtu = mtu;

        check_error(unsafe {
            libc::setsockopt(
                self.inner.as_raw_fd(),
                bluetooth_sys::SOL_L2CAP as i32,
                0x01, /* L2CAP_OPTIONS */
                &options as *const bluetooth_sys::l2cap_options as *const libc::c_void,
                len,
            )
        })?;

        Ok(())
    }

    pub fn set_nonblocking(&self, nonblocking: bool) -> std::io::Result<()> {
        self.inner.set_nonblocking(nonblocking)
    }

    pub fn local_addr(&self) -> Result<(Address, u16), std::io::Error> {
        let mut addr: SockAddr = unsafe { std::mem::zeroed() };
        let mut addr_len = match self.proto {
            BtProto::L2CAP => std::mem::size_of::<bluetooth_sys::sockaddr_l2>(),
            BtProto::RFCOMM => std::mem::size_of::<bluetooth_sys::sockaddr_rc>(),
            _ => unreachable!(),
        } as u32;

        check_error(unsafe {
            libc::getsockname(
                self.inner.as_raw_fd(),
                &mut addr as *mut _ as *mut _,
                &mut addr_len,
            )
        })?;

        let addr = match self.proto {
            BtProto::L2CAP => unsafe { (addr.l2.l2_bdaddr.into(), addr.l2.l2_psm) },
            BtProto::RFCOMM => unsafe { (addr.rc.rc_bdaddr.into(), addr.rc.rc_channel as u16) },
            _ => unreachable!(),
        };

        Ok(addr)
    }

    pub fn peer_addr(&self) -> Result<(Address, u16), std::io::Error> {
        let mut addr: SockAddr = unsafe { std::mem::zeroed() };
        let mut addr_len = match self.proto {
            BtProto::L2CAP => std::mem::size_of::<bluetooth_sys::sockaddr_l2>(),
            BtProto::RFCOMM => std::mem::size_of::<bluetooth_sys::sockaddr_rc>(),
            _ => unreachable!(),
        } as u32;

        check_error(unsafe {
            libc::getpeername(
                self.inner.as_raw_fd(),
                &mut addr as *mut _ as *mut _,
                &mut addr_len,
            )
        })?;

        let addr = match self.proto {
            BtProto::L2CAP => unsafe { (addr.l2.l2_bdaddr.into(), addr.l2.l2_psm) },
            BtProto::RFCOMM => unsafe { (addr.rc.rc_bdaddr.into(), addr.rc.rc_channel as u16) },
            _ => unreachable!(),
        };

        Ok(addr)
    }
}

impl AsRawFd for BluetoothStream {
    fn as_raw_fd(&self) -> RawFd {
        self.inner.as_raw_fd()
    }
}

impl IntoRawFd for BluetoothStream {
    fn into_raw_fd(self) -> RawFd {
        self.inner.into_raw_fd()
    }
}

impl Read for BluetoothStream {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.inner.read(buf)
    }
}

impl<'a> Read for &'a BluetoothStream {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        (&(*self).inner).read(buf)
    }
}

impl Write for BluetoothStream {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.inner.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
    }
}

impl<'a> Write for &'a BluetoothStream {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        (&(*self).inner).write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        (&(*self).inner).flush()
    }
}