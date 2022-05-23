//! This example allows you to chat over L2CAP with another bluetooth device.
//!
//! Copyright (c) 2022 Ibiyemi Abiodun

extern crate bluez;

use std::io::BufRead;

use anyhow::Context;
use bluez::communication::stream::BluetoothStream;

use bluez::socket::BtProto;
use bluez::Address;
use bluez::AddressType;
use clap::Parser;
use tokio::io::BufReader;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt};

#[derive(Parser, Debug)]
struct Args {
    address: Address,
    port: u16,
}

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

    let args = Args::parse();

    let stream =
        BluetoothStream::connect(BtProto::L2CAP, args.address, AddressType::BREDR, args.port)
            .await?;

    println!(
        "l2cap client connected to {} on port {}",
        args.address, args.port
    );

    let (reader, mut writer) = tokio::io::split(stream);
    let mut reader = BufReader::new(reader);

    {
        let read_fut = {
            async {
                let mut line = String::new();

                while reader.read_line(&mut line).await? > 0 {
                    println!("> {}", line);
                    line.clear();
                }

                #[allow(unreachable_code)]
                Ok::<_, anyhow::Error>(())
            }
        };

        let write_fut = {
            async {
                loop {
                    let line = input_rx.recv().await.context("stdin ended")?;
                    writer.write(line.as_bytes()).await?;
                    // writer.flush().await?;
                    println!("< {}", line);
                }

                #[allow(unreachable_code)]
                Ok::<_, anyhow::Error>(())
            }
        };

        futures::pin_mut!(read_fut);
        futures::pin_mut!(write_fut);
        let _ = futures::future::join(read_fut, write_fut).await;
    }

    writer.shutdown().await?;

    Ok(())
}
