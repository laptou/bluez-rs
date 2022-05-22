use std::os::unix::net::UnixStream as StdUnixStream;

use std::u16;

use bytes::*;
use libc;
use std::os::unix::io::{FromRawFd, RawFd};
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;

use crate::management::interface::{Request, Response};
use crate::management::Error;
use crate::socket::*;

#[derive(Debug)]
pub struct ManagementStream(
    // reads need to be buffered so that methods like read_exact do not end up
    // dropping data and writes cannot be buffered so that we don't have to
    // worry about flushing them
    BufReader<UnixStream>,
);

impl ManagementStream {
    pub fn open() -> Result<Self, std::io::Error> {
        let fd: RawFd = unsafe {
            libc::socket(
                libc::AF_BLUETOOTH,
                libc::SOCK_RAW | libc::SOCK_CLOEXEC | libc::SOCK_NONBLOCK,
                BtProto::HCI as libc::c_int,
            )
        };

        if fd < 0 {
            return Err(std::io::Error::last_os_error());
        }

        let addr = bluetooth_sys::sockaddr_hci {
            hci_family: libc::AF_BLUETOOTH as u16,
            hci_dev: bluetooth_sys::HCI_DEV_NONE as u16,
            hci_channel: bluetooth_sys::HCI_CHANNEL_CONTROL as u16,
        };

        if unsafe {
            libc::bind(
                fd,
                &addr as *const bluetooth_sys::sockaddr_hci as *const libc::sockaddr,
                std::mem::size_of::<bluetooth_sys::sockaddr_hci>() as u32,
            )
        } < 0
        {
            let err = std::io::Error::last_os_error();

            unsafe {
                libc::close(fd);
            }

            return Err(err);
        }

        Ok(ManagementStream(BufReader::new(UnixStream::from_std(
            unsafe { StdUnixStream::from_raw_fd(fd) },
        )?)))
    }

    /// Returns either an error or the number of bytes that were sent.
    pub async fn send(&mut self, request: Request) -> Result<usize, std::io::Error> {
        let buf: Bytes = request.into();
        self.0.write(&buf).await
    }

    pub async fn receive(&mut self) -> Result<Response, Error> {
        // read 6 byte header
        let mut header = [0u8; 6];
        self.0.read_exact(&mut header).await?;

        let param_size = u16::from_le_bytes([header[4], header[5]]) as usize;

        // read rest of message
        let mut body = vec![0u8; param_size];
        self.0.read_exact(&mut body[..]).await?;

        // make buffer by chaining header and body
        Response::parse(Buf::chain(&header[..], &body[..]))
    }

    // fn pin_get_inner(self: Pin<&mut Self>) -> Pin<&mut UnixStream> {
    //     unsafe { self.map_unchecked_mut(|s| &mut s.0) }
    // }
}

// impl AsyncWrite for ManagementStream {
//     fn poll_write(
//         self: Pin<&mut Self>,
//         cx: &mut Context<'_>,
//         buf: &[u8],
//     ) -> Poll<Result<usize, std::io::Error>> {
//         AsyncWrite::poll_write(self.pin_get_inner(), cx, buf)
//     }

//     fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), std::io::Error>> {
//         AsyncWrite::poll_flush(self.pin_get_inner(), cx)
//     }

//     fn poll_shutdown(
//         self: Pin<&mut Self>,
//         cx: &mut Context<'_>,
//     ) -> Poll<Result<(), std::io::Error>> {
//         AsyncWrite::poll_shutdown(self.pin_get_inner(), cx)
//     }
// }

// impl AsyncRead for ManagementStream {
//     fn poll_read(
//         self: Pin<&mut Self>,
//         cx: &mut Context<'_>,
//         buf: &mut tokio::io::ReadBuf<'_>,
//     ) -> Poll<std::io::Result<()>> {
//         AsyncRead::poll_read(self.pin_get_inner(), cx, buf)
//     }
// }
