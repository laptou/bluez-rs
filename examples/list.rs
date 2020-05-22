//! This example just gets all of the available controllers
//! on the system and spits out information about them.
//!
//! Copyright (c) 2020 Ibiyemi Abiodun

extern crate bluez;

use std::error::Error;

use bluez::client::*;

#[async_std::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
    let mut client = BlueZClient::new().unwrap();

    let version = client.get_mgmt_version().await?;
    println!(
        "management version: {}.{}",
        version.version, version.revision
    );

    let controllers = client.get_ext_controller_list().await?;

    println!("\navailable controllers:");

    for (controller, controller_type, controller_bus) in controllers {
        println!(
            "\t{:?} ({:?}, {:?})",
            controller, controller_type, controller_bus
        );
        let info = client.get_controller_info(controller).await?;

        println!("\t\tname: {:?}", info.name);
        println!("\t\tshort name: {:?}", info.short_name);
        println!("\t\taddress: {:?}", info.address);
        println!("\t\tsupported settings: {:?}", info.supported_settings);
        println!("\t\tcurrent settings: {:?}", info.current_settings);
        println!("\t\tmanufacturer: 0x{:04x}", info.manufacturer);
        println!("\t\tbluetooth version: 0x{:02x}", info.bluetooth_version);
        println!("\t\tclass of device: {:?}", info.class_of_device);
    }

    Ok(())
}
