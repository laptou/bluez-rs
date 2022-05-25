//! This example allows you to query a device for service discovery.
//!
//! Copyright (c) 2022 Ibiyemi Abiodun

extern crate bluez;

use anyhow::Context;
use bluez::communication::discovery::{
    ServiceAttributeId, ServiceAttributeRange, ServiceDiscoveryClient, SDP_BROWSE_ROOT,
};
use bluez::Address;
use clap::Parser;

#[derive(Parser, Debug)]
struct Args {
    address: Address,
}

#[tokio::main(flavor = "current_thread")]
pub async fn main() -> Result<(), anyhow::Error> {
    let args = Args::parse();

    let address = args.address;

    let mut client = ServiceDiscoveryClient::connect(address)
        .await
        .context("could not connect to device")?;

    println!("connected to device");

    // get a list of services that are available on this device by querying the browse root

    let response = client
        .service_search(vec![SDP_BROWSE_ROOT.into()], 30)
        .await
        .context("service search request failed")?;

    for service_handle in response.service_record_handles {
        println!(
            "getting attribute values for service {:#010x}",
            service_handle
        );

        // get all of the attributes for each service that was revealed

        let mut response = client
            .service_attribute(service_handle, u16::MAX, vec![ServiceAttributeRange::ALL])
            .await
            .context("service attribute request failed")?;

        // pretty-print information about each service

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
