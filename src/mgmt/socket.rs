use std::convert::{TryFrom, TryInto};
use std::ffi::CStr;
use std::os::raw::c_ushort;
use std::u16;

use async_std::io::{self, BufReader, Read, Write};
use async_std::os::unix::io::{FromRawFd, RawFd};
use async_std::os::unix::net::UnixStream;
use bytes::*;
use bytes::buf::BufExt;
use futures::{AsyncReadExt, AsyncWriteExt};
use futures::io::{ReadHalf, WriteHalf};
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
    User = 1,
    Monitor = 2,
    Control = 3,
}

const HCI_DEV_NONE: c_ushort = 65535;

#[derive(Debug)]
pub struct ManagementSocket {
    reader: BufReader<ReadHalf<UnixStream>>,
    writer: WriteHalf<UnixStream>,
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

            unsafe {
                libc::close(fd);
            }

            return Err(err);
        }

        let stream: UnixStream = unsafe { UnixStream::from_raw_fd(fd) };
        let (read_stream, write_stream) = stream.split();

        Ok(ManagementSocket {
            reader: BufReader::new(read_stream),
            writer: write_stream,
        })
    }

    /// Returns either an error or the number of bytes that were sent.
    pub async fn send(&mut self, request: ManagementRequest) -> Result<usize, io::Error> {
        let buf: Bytes = request.into();
        self.writer.write(&buf).await
    }

    pub async fn receive(&mut self) -> Result<ManagementResponse, ManagementError> {
        // read 6 byte header
        let mut header = [0u8; 6];
        self.reader.read_exact(&mut header).await?;

        // this ugliness forces a &[u8] into [u8; 2]
        let param_size = u16::from_le_bytes([header[4], header[5]]) as usize;

        // read rest of message
        let mut body = vec![0u8; param_size];
        self.reader.read_exact(&mut body[..]).await?;

        // make buffer by chaining header and body
        ManagementResponse::parse((&header[..]).chain(&body[..]))
    }
}
