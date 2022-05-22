//! This example allows you to chat over L2CAP with another bluetooth device.
//!
//! Copyright (c) 2021 Ibiyemi Abiodun

extern crate bluez;

use anyhow::Context;
use bluez::communication::discovery::{
    SdpClient, ServiceAttributeId, ServiceAttributeRange, SDP_BROWSE_ROOT,
};
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

        let mut response = client
            .service_attribute(service_handle, u16::MAX, vec![ServiceAttributeRange::ALL])
            .await
            .context("service attribute request failed")?;

        response
            .attributes
            .remove(&ServiceAttributeId::SERVICE_RECORD_HANDLE);

        if let Some(val) = response
            .attributes
            .remove(&ServiceAttributeId::SERVICE_CLASS_ID_LIST)
        {
            println!("\tservice class id list: {:?}", val)
        }

        if let Some(val) = response
            .attributes
            .remove(&ServiceAttributeId::SERVICE_RECORD_STATE)
        {
            println!("\tservice record state: {:?}", val)
        }

        if let Some(val) = response.attributes.remove(&ServiceAttributeId::SERVICE_ID) {
            println!("\tservice id: {:?}", val)
        }

        if let Some(val) = response
            .attributes
            .remove(&ServiceAttributeId::PROTOCOL_DESCRIPTOR_LIST)
        {
            println!("\tprotocol descriptor list: {:?}", val)
        }

        if let Some(val) = response
            .attributes
            .remove(&ServiceAttributeId::BROWSE_GROUP_LIST)
        {
            println!("\tbrowse group list: {:?}", val)
        }

        if let Some(val) = response
            .attributes
            .remove(&ServiceAttributeId::LANGUAGE_BASE_ATTRIBUTE_ID_LIST)
        {
            println!("\tlanguage base attribute id list: {:?}", val)
        }

        if let Some(val) = response
            .attributes
            .remove(&ServiceAttributeId::SERVICE_INFO_TIME_TO_LIVE)
        {
            println!("\tservice info ttl: {:?}", val)
        }

        if let Some(val) = response
            .attributes
            .remove(&ServiceAttributeId::SERVICE_AVAILABILITY)
        {
            println!("\tservice availability: {:?}", val)
        }

        if let Some(val) = response
            .attributes
            .remove(&ServiceAttributeId::BLUETOOTH_PROFILE_DESCRIPTOR_LIST)
        {
            println!("\tbluetooth profile descriptor list: {:?}", val)
        }

        if let Some(val) = response
            .attributes
            .remove(&ServiceAttributeId::ADDITIONAL_PROTOCOL_DESCRIPTOR_LISTS)
        {
            println!("\tadditional profile descriptor lists: {:?}", val)
        }

        if response.attributes.len() > 0 {
            println!("\tother attributes: {:?}", response.attributes);
        }
    }

    Ok(())
}
