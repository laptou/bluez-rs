use std::io;
use std::os::raw::c_ushort;
use std::os::unix::io::RawFd;

use crate::mgmt::ManagementError;

use super::interface::{ManagementRequest, ManagementResponse};

#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct sockaddr_hci {
    pub hci_family: c_ushort,
    pub hci_dev: c_ushort,
    pub hci_channel: c_ushort,
}

#[allow(unused)]
const BTPROTO_L2CAP: c_ushort = 0;
const BTPROTO_HCI: c_ushort = 1;
#[allow(unused)]
const BTPROTO_RFCOMM: c_ushort = 3;
#[allow(unused)]
const BTPROTO_AVDTP: c_ushort = 7;

const HCI_DEV_NONE: c_ushort = 65535;
#[allow(unused)]
const HCI_CHANNEL_RAW: c_ushort = 0;
const HCI_CHANNEL_CONTROL: c_ushort = 3;

#[derive(Debug)]
pub struct ManagementSocket {
    fd: RawFd,
    epoll_fd: RawFd,
    is_open: bool,
}

impl ManagementSocket {
    pub fn new() -> Result<Self, io::Error> {
        let fd = unsafe {
            libc::socket(
                libc::PF_BLUETOOTH,
                libc::SOCK_RAW | libc::SOCK_CLOEXEC | libc::SOCK_NONBLOCK,
                BTPROTO_HCI as libc::c_int,
            )
        };

        if fd < 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(ManagementSocket {
                fd,
                epoll_fd: epoll::create(true)?,
                is_open: false,
            })
        }
    }

    pub fn open(&mut self) -> Result<(), io::Error> {
        let addr = sockaddr_hci {
            hci_family: libc::AF_BLUETOOTH as u16,
            hci_dev: HCI_DEV_NONE,
            hci_channel: HCI_CHANNEL_CONTROL,
        };

        // do not open twice
        if self.is_open {
            return Ok(());
        }

        if unsafe {
            libc::bind(
                self.fd,
                &addr as *const sockaddr_hci as *const libc::sockaddr,
                core::mem::size_of::<sockaddr_hci>() as u32,
            )
        } < 0
        {
            let err = Err(io::Error::last_os_error());

            unsafe {
                libc::close(self.fd);
            }

            err
        } else {
            self.is_open = true;

            epoll::ctl(
                self.epoll_fd,
                epoll::ControlOptions::EPOLL_CTL_ADD,
                self.fd,
                epoll::Event {
                    events: (libc::EPOLLIN | libc::EPOLLRDHUP | libc::EPOLLERR | libc::EPOLLHUP)
                        as u32,
                    data: 0,
                },
            )
        }
    }

    pub fn close(&mut self) {
        unsafe {
            libc::close(self.fd);
            libc::close(self.epoll_fd);
        };
        self.is_open = false;
    }

    pub(crate) fn send(&self, request: ManagementRequest) -> Result<(), failure::Error> {
        let buf = unsafe { request.get_buf() };

        if unsafe {
            libc::write(
                self.fd,
                buf.as_slice() as *const [u8] as *const ::std::os::raw::c_void,
                buf.len(),
            )
        } < 0
        {
            Err(io::Error::last_os_error().into())
        } else {
            Ok(())
        }
    }

    pub fn receive(&self, timeout: i32) -> Result<ManagementResponse, failure::Error> {
        const BUF_SIZE: usize = 1024;
        let mut buf: Vec<u8> = vec![0; BUF_SIZE];
        let mut bytes_read = 0;

        if timeout != 0 {
            let mut events: [epoll::Event; 1] = unsafe { ::std::mem::uninitialized() };
            let event_count = epoll::wait(self.epoll_fd, timeout, &mut events[..]);

            match event_count {
                Err(e) => return Err(e.into()),
                Ok(count) => {
                    if count == 0 {
                        return Err(ManagementError::TimedOut.into());
                    }

                    let event = &events[0];
                    if event.events as i32 & libc::EPOLLIN != libc::EPOLLIN {
                        // TODO: handle fd being closed unexpectedly
                        return Err(ManagementError::Unknown.into());
                    }
                },
            }
        }

        loop {
            let result = unsafe {
                libc::read(
                    self.fd,
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
