use std::ffi::CStr;
use std::io::{self, Read, Write};
use std::os::raw::c_ushort;
use std::os::unix::io::{FromRawFd, RawFd};
use std::os::unix::net::UnixStream;
use std::u16;

use bitflags::_core::ffi::c_void;
use bytes::{Buf, Bytes, BytesMut, IntoBuf};
use libc;
use num_traits::FromPrimitive;

use crate::mgmt::interface::event::ManagementEvent;
use crate::mgmt::ManagementError;
use crate::util::get_string;

use super::interface::{ManagementRequest, ManagementResponse};

#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct SockAddrHci {
    pub hci_family: c_ushort,
    pub hci_dev: c_ushort,
    pub hci_channel: HciChannel,
}

#[repr(u16)]
#[derive(Debug, Copy, Clone)]
enum BtProto {
    L2CAP = 0,
    HCI = 1,
    RFCOMM = 3,
    AVDTP = 7,
}

#[repr(u16)]
#[derive(Debug, Copy, Clone)]
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
    fd: RawFd
}

impl ManagementSocket {
    pub fn open() -> Result<Self, io::Error> {
        let fd: RawFd = unsafe {
            libc::socket(
                libc::AF_BLUETOOTH,
                libc::SOCK_RAW | libc::SOCK_CLOEXEC | libc::SOCK_NONBLOCK,
                BtProto::HCI as libc::c_int,
            )
        };

        if fd < 0 {
            return Err(io::Error::last_os_error());
        }

        let addr = SockAddrHci {
            hci_family: libc::AF_BLUETOOTH as u16,
            hci_dev: HCI_DEV_NONE,
            hci_channel: HciChannel::Control,
        };

        if unsafe {
            libc::bind(
                fd,
                &addr as *const SockAddrHci as *const libc::sockaddr,
                std::mem::size_of::<SockAddrHci>() as u32,
            )
        } < 0
        {
            let err = io::Error::last_os_error();

            unsafe { libc::close(fd); }

            return Err(err);
        }

        Ok(ManagementSocket {
            fd
        })
    }

    pub async fn send(&mut self, request: ManagementRequest) -> Result<(), io::Error> {
        let buf: Bytes = request.into();

        if unsafe { libc::send(self.fd, buf.as_ptr() as *const c_void, buf.len(), 0) } < 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(())
        }
    }

    pub async fn receive(&mut self, timeout: i32) -> Result<ManagementResponse, ManagementError> {
        let mut header = [0u8; 6];

        // need MSG_PEEK otherwise recv() clears the socket and we get EAGAIN when
        // we try to read the socket again; this is the main reason for using a raw
        // fd instead of a UnixStream
        if unsafe { libc::recv(self.fd, header.as_mut_ptr() as *mut c_void, 6, libc::MSG_PEEK) } < 0 {
            return Err(io::Error::last_os_error().into());
        }

        let mut cursor = header.into_buf();

        let evt_code = cursor.get_u16_le();
        let controller = cursor.get_u16_le();
        let param_size = cursor.get_u16_le() as usize;

        let mut param = vec![0u8; param_size + 6];

        if unsafe { libc::recv(self.fd, param.as_mut_ptr() as *mut c_void, param.len(), 0) } < 0 {
            return Err(io::Error::last_os_error().into());
        }

        let mut cursor = param.into_buf();

        cursor.advance(6);

        return Ok(ManagementResponse {
            controller,
            event: match evt_code {
                0x0001 | 0x0002 => {
                    let opcode = cursor.get_u16_le();
                    let opcode = FromPrimitive::from_u16(opcode)
                        .ok_or(ManagementError::UnknownOpcode { opcode })?;

                    let status = cursor.get_u8();
                    let status = FromPrimitive::from_u8(status)
                        .ok_or(ManagementError::UnknownStatus { status })?;

                    if evt_code == 0x0001 {
                        ManagementEvent::CommandComplete {
                            opcode,
                            status,
                            param: cursor.collect(),
                        }
                    } else {
                        ManagementEvent::CommandStatus {
                            opcode,
                            status,
                        }
                    }
                }
                0x0003 => ManagementEvent::ControllerError { code: cursor.get_u8() },
                0x0004 => ManagementEvent::IndexAdded,
                0x0005 => ManagementEvent::IndexRemoved,
                0x0006 => ManagementEvent::NewSettings {
                    settings: unimplemented!(),
                },
                0x0007 => ManagementEvent::ClassOfDeviceChanged {
                    class: unimplemented!(),
                },
                0x0008 => {
                    let name = get_string(&mut cursor, 249);
                    let short_name = get_string(&mut cursor, 11);

                    ManagementEvent::LocalNameChanged { name, short_name }
                }
                0x0009 => unimplemented!(),
                0x000A => unimplemented!(),
                0x000B => unimplemented!(),
                0x000C => unimplemented!(),
                0x000D => unimplemented!(),
                0x000E => unimplemented!(),
                0x000F => unimplemented!(),
                0x0010 => unimplemented!(),
                0x0011 => unimplemented!(),
                0x0012 => unimplemented!(),
                0x0013 => unimplemented!(),
                0x0014 => unimplemented!(),
                0x0015 => unimplemented!(),
                0x0016 => unimplemented!(),
                0x0017 => unimplemented!(),
                _ => unreachable!(),
            },
        });
    }

    pub fn close(&mut self) {
        unsafe { libc::close(self.fd); }
    }
}