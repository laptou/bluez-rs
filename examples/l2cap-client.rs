//! This example allows you to chat over L2CAP with another bluetooth device.
//!
//! Copyright (c) 2021 Ibiyemi Abiodun

extern crate bluez;

use bluez::communication::stream::BluetoothStream;

use bluez::socket::BtProto;
use bluez::Address;
use bluez::AddressType;
use tokio::io::BufReader;
use tokio::io::BufWriter;
use tokio::io::{stdin, stdout, AsyncBufReadExt, AsyncWriteExt};
use tokio::spawn;

#[tokio::main(flavor = "current_thread")]
pub async fn main() -> Result<(), anyhow::Error> {
    print!("enter l2cap server address: ");
    stdout().flush().await?;
    let mut line = String::new();
    let mut stdin = BufReader::new(stdin());
    stdin.read_line(&mut line).await?;

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
    stdin.read_line(&mut line).await?;

    let port = line.trim().parse()?;

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
            let mut line = String::new();
            loop {
                stdin.read_line(&mut line).await?;
                writer.write(line.as_bytes()).await?;
                writer.flush().await?;
                println!("< {}", line);
                line.clear();
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
