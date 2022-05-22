//! This example allows you to chat over L2CAP with another bluetooth device.
//!
//! Copyright (c) 2021 Ibiyemi Abiodun

extern crate bluez;

use std::error::Error;
use std::io::{stdin, stdout, Write};
use std::sync::Arc;

// use bluez::communication::discovery::{SdpAttributeSpecification, SdpSession, SdpSessionFlags};
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

    let session = SdpSession::connect(
        Address::zero(),
        address,
        SdpSessionFlags::RetryIfBusy.into(),
    )?;

    // let records = session.search_req(&[0x110Bu32.into()], 30)?;

    // for record in records {
    //     println!("0x{:04x?}", record);
    // }

    let records = session.search_attr_req(
        &[0x110Bu32.into()],
        SdpAttributeSpecification::Range,
        &[0x0000FFFF],
    )?;

    for record in records {
        println!("handle -> 0x{:08x?}", record.handle());

        for data in record.get_access_protos()? {
            println!("{:?}", data.value());
        }
    }

    Ok(())
}
