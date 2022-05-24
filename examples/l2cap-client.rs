//! This example allows you to chat over L2CAP with another bluetooth device.
//!
//! Copyright (c) 2022 Ibiyemi Abiodun

extern crate bluez;

use std::io::BufRead;

use anyhow::Context;
use bluez::communication::stream::BluetoothStream;

use bluez::Address;
use bluez::AddressType;
use bluez::Protocol;
use clap::Parser;
use tokio::io::BufReader;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt};
use tokio::spawn;

#[derive(Parser, Debug)]
struct Args {
    address: Address,
    port: u16,
}

#[tokio::main(worker_threads = 2)]
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
        BluetoothStream::connect(Protocol::L2CAP, args.address, AddressType::BREDR, args.port)
            .await?;

    println!(
        "l2cap client connected to {} on port {}",
        args.address, args.port
    );

    // note: using tokio::io::split for this use-case does not work, because that method
    // uses a lock internally, whereas this one does not
    let (reader, mut writer) = stream.into_split();

    let read_task = spawn(async move {
        let mut line = String::new();
        let mut reader = BufReader::new(reader);

        while reader.read_line(&mut line).await.unwrap() > 0 {
            println!("> {}", line);
            line.clear();
        }
    });

    let write_task = spawn(async move {
        loop {
            let line = input_rx.recv().await.context("stdin ended").unwrap();
            writer.write_all(line.as_bytes()).await.unwrap();
            println!("< {}", line);
        }
    });

    read_task.await?;
    write_task.abort();

    Ok(())
}
