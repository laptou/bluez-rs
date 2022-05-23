//! This example allows you to chat over L2CAP with another bluetooth device.
//!
//! Copyright (c) 2021 Ibiyemi Abiodun

extern crate bluez;

use std::io::BufRead;
use std::io::Write;
use std::str::FromStr;

use anyhow::Context;
use bluez::communication::stream::BluetoothStream;

use bluez::socket::BtProto;
use bluez::Address;
use bluez::AddressType;
use tokio::io::BufReader;
use tokio::io::BufWriter;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt};
use tokio::spawn;

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

    print!("enter l2cap server address: ");
    std::io::stdout().flush()?;
    let address = input_rx
        .recv()
        .await
        .context("server address is required")?;
    let address = Address::from_str(address.trim())?;

    print!("enter l2cap server port: ");
    std::io::stdout().flush()?;
    let port = input_rx.recv().await.context("server port is required")?;
    let port = port.trim().parse()?;

    let stream =
        BluetoothStream::connect(BtProto::L2CAP, address, AddressType::BREDR, port).await?;

    println!("l2cap client connected to {} on port {}", address, port);

    let (read, write) = tokio::io::split(stream);

    let read_task = spawn({
        async move {
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
    });

    let write_task = spawn({
        async move {
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
    });

    let (res1, res2) = futures::join!(read_task, write_task);
    res1??;
    res2??;

    Ok(())
}
