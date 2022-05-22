//! This example allows you to chat over L2CAP with another bluetooth device.
//!
//! Copyright (c) 2021 Ibiyemi Abiodun

extern crate bluez;

use anyhow::Context;
use bluez::communication::discovery::{AttributeRange, SdpClient, SDP_BROWSE_ROOT};
use bluez::Address;

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

    let mut client = SdpClient::connect(address)
        .await
        .context("could not connect to device")?;

    println!("connected to device");

    let response = client
        .service_search(vec![SDP_BROWSE_ROOT.into()], 30)
        .await
        .context("service search request failed")?;

    for service_handle in response.service_record_handles {
        println!(
            "getting attribute values for service {:#010x}",
            service_handle
        );

        let response = client
            .service_attribute(service_handle, u16::MAX, vec![AttributeRange::ALL])
            .await
            .context("service attribute request failed")?;

        println!("{:?}", response.attributes);
    }

    Ok(())
}
