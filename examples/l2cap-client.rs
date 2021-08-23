//! This example allows you to chat over L2CAP with another bluetooth device.
//!
//! Copyright (c) 2021 Ibiyemi Abiodun

extern crate bluez;

use std::error::Error;
use std::sync::Arc;

use async_std::io::{stdin, stdout};
use bluez::communication::socket::{L2capListener, L2capStream};
use bluez::management::client::*;
use bluez::Address;
use futures::AsyncReadExt;
use smol::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter};
use smol::Async;

#[async_std::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
    print!("enter l2cap server address: ");
    stdout().flush().await?;
    let mut line = String::new();
    stdin().read_line(&mut line).await?;

    let octets = line
        .trim()
        .split(':')
        .map(|octet| u8::from_str_radix(octet, 16))
        .rev()
        .collect::<Result<Vec<_>, _>>()?;

    let address = Address::from_slice(&octets[..]);

    print!("enter l2cap server port: ");
    stdout().flush().await?;
    let mut line = String::new();
    stdin().read_line(&mut line).await?;

    let port = line.trim().parse()?;

    let stream = L2capStream::connect(address, AddressType::BREDR, port)?;

    println!("l2cap client connected to {} on port {}", address, port);

    let stream = Arc::new(Async::new(stream)?);

    let read_task = smol::spawn({
        let sock = stream.clone();
        async move {
            let mut reader = BufReader::new(sock.as_ref());
            let mut line = String::new();
            loop {
                reader.read_line(&mut line).await?;
                println!("> {}", line);
                line.clear();
            }

            std::io::Result::Ok(())
        }
    });

    let write_task = smol::spawn({
        let sock = stream.clone();

        async move {
            let mut writer = BufWriter::new(sock.as_ref());
            let mut line = String::new();
            let stdin = stdin();
            loop {
                stdin.read_line(&mut line).await?;
                writer.write(line.as_bytes()).await?;
                writer.flush().await?;
                println!("< {}", line);
                line.clear();
            }

            std::io::Result::Ok(())
        }
    });

    let (res1, res2) = futures::join!(read_task, write_task);
    res1?;
    res2?;

    Ok(())
}