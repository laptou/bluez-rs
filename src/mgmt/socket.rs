use std::io;
use std::os::raw::c_ushort;
use std::os::unix::io::{RawFd, UnixStream};

use bytes::BytesMut;

use crate::mgmt::ManagementError;

use super::interface::{ManagementRequest, ManagementResponse};

#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct SockAddrHci {
    pub hci_family: c_ushort,
    pub hci_dev: c_ushort,
    pub hci_channel: HciChannel,
}

#[repr(u16)]
enum BtProto {
    L2CAP = 0,
    HCI = 1,
    RFCOMM = 3,
    AVDTP = 7,
}

#[repr(u16)]
enum HciChannel {
    Raw = 0,
    Control = 3,
}

const HCI_DEV_NONE: c_ushort = 65535;

/// A wrapper over the raw libc socket
/// We can't use Rust's UnixSocket because it only accepts paths
/// and we can't connect to BlueZ using a normal path
#[derive(Debug)]
pub struct ManagementSocket {
    socket: UnixStream
}

impl ManagementSocket {
    pub fn open() -> Result<Self, io::Error> {
        let fd: RawFd = unsafe {
            libc::socket(
                libc::PF_BLUETOOTH,
                libc::SOCK_RAW | libc::SOCK_CLOEXEC | libc::SOCK_NONBLOCK,
                BtProto::HCI as libc::c_int,
            )
        };

        if fd < 0 {
            return Err(io::Error::last_os_error());
        }

        if unsafe {
            libc::bind(
                self.socket_fd,
                &addr as *const SockAddrHci as *const libc::sockaddr,
                core::mem::size_of::<SockAddrHci>() as u32,
            )
        } < 0
        {
            let err = Err(io::Error::last_os_error());

            unsafe { libc::close(fd); }

            err
        }

        let addr = SockAddrHci {
            hci_family: libc::AF_BLUETOOTH as u16,
            hci_dev: HCI_DEV_NONE,
            hci_channel: HciChannel::Control,
        };

            Ok(ManagementSocket {
                socket: fd.into()
            })
    }

    pub fn open(&mut self) -> Result<(), io::Error> {


        // do not open twice
        if self.is_open {
            return Ok(());
        }

        if unsafe {
            libc::bind(
                self.socket_fd,
                &addr as *const SockAddrHci as *const libc::sockaddr,
                core::mem::size_of::<SockAddrHci>() as u32,
            )
        } < 0
        {
            let err = Err(io::Error::last_os_error());

            unsafe {
                libc::close(self.socket_fd);
            }

            err
        } else {
            self.is_open = true;

            // create epoll listener
            epoll::ctl(
                self.epoll_fd,
                epoll::ControlOptions::EPOLL_CTL_ADD,
                self.socket_fd,
                epoll::Event {
                    events: (libc::EPOLLIN | libc::EPOLLRDHUP | libc::EPOLLERR | libc::EPOLLHUP)
                        as u32,
                    data: 0,
                })
        }
    }

    pub fn close(&mut self) {
        unsafe {
            libc::close(self.socket_fd);
            libc::close(self.epoll_fd);
        };

        self.is_open = false;
    }

    pub async fn send(&self, request: ManagementRequest) -> Result<(), io::Error> {
        let buf = request.into();

        if unsafe {
            libc::write(
                self.socket_fd,
                buf.as_ptr() as *const ::std::os::raw::c_void,
                buf.len(),
            )
        } < 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(())
        }
    }

    pub async fn receive(&self, timeout: i32) -> Result<ManagementResponse, io::Error> {
        let mut buf = BytesMut::new();
        let mut bytes_read = 0;

        loop {
            let result = unsafe {
                libc::read(
                    self.socket_fd,
                    buf.as_mut_ptr() as *mut ::std::os::raw::c_void,
                    BUF_SIZE,
                )
            };

            if result >= 0 {
                bytes_read += result as usize;

                if result == BUF_SIZE as isize {
                    buf.resize(buf.len() + BUF_SIZE, 0);
                } else {
                    break;
                }
            } else {
                return Err(io::Error::last_os_error().into());
            }
        }

        buf.truncate(bytes_read);

        unsafe { ManagementResponse::from_buf(&buf) }
    }
}

impl Drop for ManagementSocket {
    fn drop(&mut self) {
        self.close()
    }
}
