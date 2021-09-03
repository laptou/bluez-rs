//! This example allows you to chat over L2CAP with another bluetooth device.
//!
//! Copyright (c) 2021 Ibiyemi Abiodun

extern crate bluez;

use std::error::Error;
use std::sync::Arc;

use async_std::io::stdin;
use bluez::communication::stream::BluetoothListener;
use bluez::management::client::*;
use bluez::socket::BtProto;
use smol::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter};
use smol::Async;

#[async_std::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
    let mut mgmt = ManagementClient::new()?;
    let controllers = mgmt.get_controller_list().await?;
    if controllers.len() < 1 {
        panic!("there are no bluetooth controllers on this device")
    }

    let controller_info = mgmt.get_controller_info(controllers[0]).await?;

    let listener = BluetoothListener::bind(
        BtProto::L2CAP,
        controller_info.address,
        AddressType::BREDR,
        0,
    )?;
    let (addr, port) = listener.local_addr()?;

    println!("l2cap server listening at {} on port {}", addr, port);

    let listener = Async::new(listener)?;

    loop {
        let (sock, (addr, port)) = listener.read_with(|l| l.accept()).await?;

        println!("l2cap server got connection from {} on port {}", addr, port);

        let sock = Arc::new(Async::new(sock)?);

        let read_task = smol::spawn({
            let sock = sock.clone();
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
            let sock = sock.clone();

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
    }
}
