//! This example allows you to chat over L2CAP with another bluetooth device.
//!
//! Copyright (c) 2021 Ibiyemi Abiodun

extern crate bluez;

use anyhow::Context;
use bluez::communication::discovery::SdpStream;
use bluez::communication::BASE_UUID;

use bluez::{Address};

#[tokio::main(flavor = "current_thread")]
pub async fn main() -> Result<(), anyhow::Error> {
    // print!("enter sdp server address: ");
    // stdout().flush()?;
    // let mut line = String::new();
    // stdin().read_line(&mut line)?;

    let octets = "94:DB:56:C0:18:1A"
        .trim()
        .split(':')
        .map(|octet| u8::from_str_radix(octet, 16))
        .rev()
        .collect::<Result<Vec<_>, _>>()?;

    let address = Address::from_slice(&octets[..]);

    let mut stream = SdpStream::connect(address)
        .await
        .context("could not connect to device")?;

    println!("connected to device");

    let response = stream
        .service_search(vec![BASE_UUID.into()], 30)
        .await
        .context("service search request failed")?;

    println!("service search response: {:?}", response);

    Ok(())
}
