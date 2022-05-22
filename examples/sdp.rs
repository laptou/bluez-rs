//! This example allows you to chat over L2CAP with another bluetooth device.
//!
//! Copyright (c) 2021 Ibiyemi Abiodun

extern crate bluez;

use std::error::Error;
use std::io::{stdin, stdout, Write};

use anyhow::Context;
use bluetooth_sys::SDP_PSM;
use bluez::communication::discovery::service_search_request;
use bluez::communication::stream::BluetoothStream;
use bluez::management::client::AddressType;
use bluez::socket::BtProto;
use bluez::Address;
use bluez::communication::BASE_UUID;

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

    let mut stream =
        BluetoothStream::connect(BtProto::L2CAP, address, AddressType::BREDR, SDP_PSM as u16)
            .await
            .context("could not connect to device")?;

    println!("connected to device");

    let response = service_search_request(&mut stream, vec![BASE_UUID.into()], 30)
        .await
        .context("service search request failed")?;

    println!("service search response: {:?}", response);

    Ok(())
}
