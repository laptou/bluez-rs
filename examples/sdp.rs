//! This example allows you to chat over L2CAP with another bluetooth device.
//!
//! Copyright (c) 2021 Ibiyemi Abiodun

extern crate bluez;

use std::error::Error;
use std::io::{stdin, stdout, Write};

use bluez::communication::discovery::service_search_request;
use bluez::communication::stream::BluetoothStream;
use bluez::management::client::AddressType;
use bluez::socket::BtProto;
use bluez::Address;

pub fn main() -> Result<(), Box<dyn Error>> {
    print!("enter sdp server address: ");
    stdout().flush()?;
    let mut line = String::new();
    stdin().read_line(&mut line)?;

    let octets = line
        .trim()
        .split(':')
        .map(|octet| u8::from_str_radix(octet, 16))
        .rev()
        .collect::<Result<Vec<_>, _>>()?;

    let address = Address::from_slice(&octets[..]);

    let mut stream = BluetoothStream::connect(BtProto::L2CAP, address, AddressType::BREDR, 0)?;

    let response = service_search_request(&mut stream, vec![0x110Bu32.into()], 30);

    println!("service search response: {:?}", response);

    Ok(())
}
