//! This example allows you to chat over L2CAP with another bluetooth device.
//!
//! Copyright (c) 2022 Ibiyemi Abiodun

extern crate bluez;

use std::{io::BufRead, sync::Arc};

use anyhow::Context;
use bluez::communication::stream::BluetoothListener;
use bluez::management::*;
use bluez::AddressType;
use bluez::Protocol;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    spawn,
    sync::Mutex,
};

#[tokio::main(worker_threads = 2)]
pub async fn main() -> Result<(), anyhow::Error> {
    let (input_tx, input_rx) = tokio::sync::mpsc::channel(16);

    std::thread::spawn(move || {
        let stdin = std::io::stdin();
        let mut stdin = stdin.lock();

        loop {
            let mut line = String::new();
            stdin.read_line(&mut line).unwrap();
            input_tx.blocking_send(line).unwrap();
        }
    });

    let mut mgmt = ManagementStream::open()?;
    let controllers = get_controller_list(&mut mgmt, None).await?;
    if controllers.len() < 1 {
        panic!("there are no bluetooth controllers on this device")
    }

    let controller_info = get_controller_info(&mut mgmt, controllers[0], None).await?;

    let listener = BluetoothListener::bind(
        Protocol::L2CAP,
        controller_info.address,
        AddressType::BREDR,
        0,
    )?;
    let (addr, port) = listener.local_addr()?;

    println!("l2cap server listening at {} on port {}", addr, port);

    let input_rx = Arc::new(Mutex::new(input_rx));

    loop {
        let (stream, (addr, port)) = listener.accept().await?;

        println!("l2cap client connected from {} on port {}", addr, port);

        let (reader, mut writer) = stream.into_split();

        let read_task = spawn(async move {
            let mut line = String::new();
            let mut reader = BufReader::new(reader);

            while reader.read_line(&mut line).await.unwrap() > 0 {
                println!("> {}", line);
                line.clear();
            }
        });

        let input_rx = input_rx.clone();

        let write_task = spawn(async move {
            let mut input_rx = input_rx.lock().await;

            loop {
                let line = input_rx.recv().await.context("stdin ended").unwrap();

                println!("< {}", line);
                writer.write_all(line.as_bytes()).await.unwrap();
                println!("<< {}", line);
            }
        });

        read_task.await?;
        write_task.abort();

        println!("l2cap client disconnected, listening again");
    }
}
