use std::mem::MaybeUninit;
use std::u16;

use bytes::*;
use futures::io::{AsyncReadExt, AsyncWriteExt, BufReader, ReadHalf, WriteHalf};
use libc;
use smol::net::unix::UnixStream;
use std::os::unix::io::{FromRawFd, RawFd};

use crate::management::client::AddressType;
use crate::management::interface::{Request, Response};
use crate::management::Error;
use crate::{socket::*, Address};

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct SockAddrL2 {
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

#[derive(Debug)]
pub struct L2capSocket {
    fd: i32,
    reader: BufReader<ReadHalf<UnixStream>>,
    writer: WriteHalf<UnixStream>,
}

impl L2capSocket {
    pub fn connect(
        addr: Address,
        addr_type: AddressType,
        port: u16,
    ) -> Result<Self, smol::io::Error> {
        let fd: RawFd = unsafe {
            libc::socket(
                libc::AF_BLUETOOTH,
                libc::SOCK_CLOEXEC | libc::SOCK_NONBLOCK | libc::SOCK_SEQPACKET,
                BtProto::L2CAP as libc::c_int,
            )
        };

        if fd < 0 {
            return Err(smol::io::Error::last_os_error());
        }

        let addr = SockAddrL2 {
            l2_family: libc::AF_BLUETOOTH as i16,
            l2_bdaddr: addr,
            l2_bdaddr_type: addr_type,
            l2_psm: port,
            ..Default::default()
        };

        if unsafe {
            libc::connect(
                fd,
                &addr as *const SockAddrL2 as *const libc::sockaddr,
                std::mem::size_of::<SockAddrL2>() as u32,
            )
        } < 0
        {
            let err = std::io::Error::last_os_error();

            unsafe {
                libc::close(fd);
            }

            return Err(err);
        }

        let stream = unsafe { std::os::unix::net::UnixStream::from_raw_fd(fd) };
        let stream = smol::Async::new(stream)?;
        let stream = UnixStream::from(stream);
        let (read_stream, write_stream) = stream.split();

        Ok(L2capSocket {
            fd,
            reader: BufReader::new(read_stream),
            writer: write_stream,
        })
    }

    pub fn set_mtu(&mut self, mtu: u16) -> std::io::Result<()> {
        let mut options = std::mem::MaybeUninit::<L2capOptions>::uninit();
        let mut len = std::mem::size_of::<L2capOptions>() as u32;

        let err = unsafe {
            libc::getsockopt(
                self.fd,
                SOL_L2CAP,
                0x01, /* L2CAP_OPTIONS */
                &mut options as *mut MaybeUninit<L2capOptions> as *mut libc::c_void,
                &mut len,
            )
        };

        if err < 0 {
            return Err(std::io::Error::last_os_error());
        }

        let mut options = unsafe { options.assume_init() };

        options.omtu = mtu;
        options.imtu = mtu;

        let err = unsafe {
            libc::setsockopt(
                self.fd,
                SOL_L2CAP,
                0x01, /* L2CAP_OPTIONS */
                &options as *const L2capOptions as *const libc::c_void,
                len,
            )
        };

        if err < 0 {
            return Err(std::io::Error::last_os_error());
        }

        Ok(())
    }

    /// Returns either an error or the number of bytes that were sent.
    pub async fn send(&mut self, request: Request) -> Result<usize, smol::io::Error> {
        let buf: Bytes = request.into();
        self.writer.write(&buf).await
    }

    pub async fn receive(&mut self) -> Result<Response, Error> {
        // read 6 byte header
        let mut header = [0u8; 6];
        self.reader.read_exact(&mut header).await?;

        // this ugliness forces a &[u8] into [u8; 2]
        let param_size = u16::from_le_bytes([header[4], header[5]]) as usize;

        // read rest of message
        let mut body = vec![0u8; param_size];
        self.reader.read_exact(&mut body[..]).await?;

        // make buffer by chaining header and body
        Response::parse(Buf::chain(&header[..], &body[..]))
    }
}
