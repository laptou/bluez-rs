//! This example just gets all of the available controllers
//! on the system and spits out information about them.
//!
//! Copyright (c) 2022 Ibiyemi Abiodun

extern crate bluez;

use anyhow::Context;
use bluez::management::*;

#[tokio::main]
pub async fn main() -> Result<(), anyhow::Error> {
    let mut mgmt = ManagementStream::open().context("failed to open management socket")?;

    let version = get_mgmt_version(&mut mgmt, None)
        .await
        .context("failed to get management api version")?;
    println!(
        "management version: {}.{}",
        version.version, version.revision
    );

    let controllers = get_ext_controller_list(&mut mgmt, None)
        .await
        .context("failed to get list of bluetooth controllers")?;

    println!("\navailable controllers:");

    for (controller, controller_type, controller_bus) in controllers {
        println!(
            "\t{:?} ({:?}, {:?})",
            controller, controller_type, controller_bus
        );
        let info = get_controller_info(&mut mgmt, controller, None)
            .await
            .context("failed to get info about controller")?;

        println!("\t\tname: {:?}", info.name);
        println!("\t\tshort name: {:?}", info.short_name);
        println!("\t\taddress: {}", info.address);
        println!("\t\tsupported settings: {:?}", info.supported_settings);
        println!("\t\tcurrent settings: {:?}", info.current_settings);
        println!("\t\tmanufacturer: 0x{:04x}", info.manufacturer);
        println!("\t\tbluetooth version: 0x{:02x}", info.bluetooth_version);
        println!("\t\tclass of device: {:?}", info.class_of_device);
    }

    Ok(())
}
