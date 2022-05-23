//! This example allows you to chat over L2CAP with another bluetooth device.
//!
//! Copyright (c) 2021 Ibiyemi Abiodun

extern crate bluez;

use std::io::BufRead;

use anyhow::Context;
use bluez::communication::stream::BluetoothListener;
use bluez::management::client::*;
use bluez::socket::BtProto;
use bluez::AddressType;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter};

#[tokio::main(flavor = "current_thread")]
pub async fn main() -> Result<(), anyhow::Error> {
    let (input_tx, mut input_rx) = tokio::sync::mpsc::channel(16);

    std::thread::spawn(move || -> anyhow::Result<()> {
        let stdin = std::io::stdin();
        let mut stdin = stdin.lock();

        loop {
            let mut line = String::new();
            stdin.read_line(&mut line)?;
            input_tx.blocking_send(line)?;
        }
    });

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

    loop {
        let (stream, (addr, port)) = listener.accept().await?;

        println!("l2cap server got connection from {} on port {}", addr, port);

        let (read, write) = tokio::io::split(stream);

        let read_fut = {
            async {
                let mut reader = BufReader::new(read);
                let mut line = String::new();
                loop {
                    reader.read_line(&mut line).await?;
                    println!("> {}", line);
                    line.clear();
                }

                #[allow(unreachable_code)]
                Ok::<_, anyhow::Error>(())
            }
        };

        let write_fut = {
            async {
                let mut writer = BufWriter::new(write);
                loop {
                    let line = input_rx.recv().await.context("stdin ended")?;
                    writer.write(line.as_bytes()).await?;
                    writer.flush().await?;
                    println!("< {}", line);
                }

                #[allow(unreachable_code)]
                Ok::<_, anyhow::Error>(())
            }
        };

        futures::pin_mut!(read_fut);
        futures::pin_mut!(write_fut);
        futures::future::select(read_fut, write_fut).await;

        println!("l2cap client disconnected, listening again");
    }
}
