use std::io::Error;
use std::mem::MaybeUninit;
use std::os::unix::net::UnixStream as StdUnixStream;

use libc;
use std::os::unix::io::{AsRawFd, FromRawFd, RawFd};
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::unix::AsyncFd;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::UnixStream;

use crate::util::check_error;
use crate::{socket::*, Address, AddressType};

union SockAddr {
    l2: bluetooth_sys::sockaddr_l2,
    rc: bluetooth_sys::sockaddr_rc,
}

pub struct BluetoothListener {
    inner: AsyncFd<RawFd>,
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
                libc::SOCK_CLOEXEC | libc::SOCK_NONBLOCK | flags,
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

        Ok(BluetoothListener {
            inner: AsyncFd::new(fd)?,
            proto,
        })
    }

    pub async fn accept(&self) -> Result<(BluetoothStream, (Address, u16)), std::io::Error> {
        let mut addr: SockAddr = unsafe { std::mem::zeroed() };
        let mut addr_len = match self.proto {
            BtProto::L2CAP => std::mem::size_of::<bluetooth_sys::sockaddr_l2>(),
            BtProto::RFCOMM => std::mem::size_of::<bluetooth_sys::sockaddr_rc>(),
            _ => unreachable!(),
        } as u32;

        let fd = loop {
            let res = self.inner.readable().await?.try_io(|_fd| {
                Ok(check_error(unsafe {
                    libc::accept(
                        self.inner.as_raw_fd(),
                        &mut addr as *mut _ as *mut _,
                        &mut addr_len,
                    )
                })?)
            });

            match res {
                Ok(fd) => break fd?,
                Err(_would_block) => continue,
            }
        };

        let addr = match self.proto {
            BtProto::L2CAP => unsafe { (addr.l2.l2_bdaddr.into(), addr.l2.l2_psm) },
            BtProto::RFCOMM => unsafe { (addr.rc.rc_bdaddr.into(), addr.rc.rc_channel as u16) },
            _ => unreachable!(),
        };

        let sock = BluetoothStream {
            inner: UnixStream::from_std(unsafe { StdUnixStream::from_raw_fd(fd) })?,
            proto: self.proto,
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
}

impl AsRawFd for BluetoothListener {
    fn as_raw_fd(&self) -> RawFd {
        self.inner.as_raw_fd()
    }
}

#[derive(Debug)]
pub struct BluetoothStream {
    inner: UnixStream,
    proto: BtProto,
}

impl BluetoothStream {
    pub async fn connect(
        proto: BtProto,
        addr: Address,
        addr_type: AddressType,
        port: u16,
    ) -> Result<Self, std::io::Error> {
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
                libc::SOCK_CLOEXEC | libc::SOCK_NONBLOCK | flags,
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

        let res = unsafe {
            libc::connect(
                fd,
                &addr as *const SockAddr as *const libc::sockaddr,
                addr_len as u32,
            )
        };

        match check_error(res) {
            Ok(_) => {}
            // should always get EINPROGRESS if socket is initialized using SOCK_NONBLOCK
            Err(err) if err.raw_os_error() == Some(libc::EINPROGRESS) => {
                // wait until the file descriptor becomes writeable
                let afd = AsyncFd::new(fd)?;
                let _ = afd.writable().await?;
            }
            other => {
                other?;
            }
        }

        Ok(BluetoothStream {
            inner: UnixStream::from_std(unsafe { StdUnixStream::from_raw_fd(fd) })?,
            proto,
        })
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

    fn pin_get_inner(self: Pin<&mut Self>) -> Pin<&mut UnixStream> {
        unsafe { self.map_unchecked_mut(|s| &mut s.inner) }
    }
}

impl AsRawFd for BluetoothStream {
    fn as_raw_fd(&self) -> RawFd {
        self.inner.as_raw_fd()
    }
}

impl AsRef<UnixStream> for BluetoothStream {
    fn as_ref(&self) -> &UnixStream {
        &self.inner
    }
}

impl AsMut<UnixStream> for BluetoothStream {
    fn as_mut(&mut self) -> &mut UnixStream {
        &mut self.inner
    }
}

impl AsyncWrite for BluetoothStream {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, Error>> {
        AsyncWrite::poll_write(self.pin_get_inner(), cx, buf)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
        AsyncWrite::poll_flush(self.pin_get_inner(), cx)
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
        AsyncWrite::poll_shutdown(self.pin_get_inner(), cx)
    }
}

impl AsyncRead for BluetoothStream {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        AsyncRead::poll_read(self.pin_get_inner(), cx, buf)
    }
}
