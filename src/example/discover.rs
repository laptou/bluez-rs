extern crate bluez;

use std::error::Error;

use async_std::task::block_on;

use bluez::client::*;
use bluez::interface::controller::*;
use bluez::interface::event::ManagementEvent;

#[async_std::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
    let mut client = ManagementClient::new().unwrap();

    let version = client.get_mgmt_version().await?;
    println!(
        "management version: {}.{}",
        version.version, version.revision
    );

    let controllers = client.get_controller_list().await?;

    // async closures aren't stable yet so we'll just block on each one
    // instead of using streams
    let controllers_info = controllers
        .into_iter()
        .map(|controller| {
            (
                controller,
                block_on(client.get_controller_info(controller)).unwrap(),
            )
        })
        .collect::<Vec<(Controller, ControllerInfo)>>();

    println!("\navailable controllers:");

    for (controller, info) in &controllers_info {
        println!("\t{:?}", controller);

        println!("\t\tname: {:?}", info.name);
        println!("\t\tshort name: {:?}", info.short_name);
        println!("\t\tsupported settings: {:?}", info.supported_settings);
        println!("\t\tcurrent settings: {:?}", info.current_settings);
        println!("\t\tmanufacturer: {:?}", info.manufacturer);
        println!("\t\tbluetooth version: {:?}", info.bluetooth_version);
        println!("\t\tclass of device: {:?}", info.class_of_device);
    }

    // find the first controller we can power on
    let (controller, info) = controllers_info
        .into_iter()
        .filter(|(_, info)| info.supported_settings.contains(ControllerSetting::Powered))
        .nth(0)
        .expect("no usable controllers found");

    if !info.current_settings.contains(ControllerSetting::Powered) {
        println!("powering on bluetooth controller {}", controller);
        client.set_powered(controller, true).await?;
    }

    // scan for some devices
    // to do this we'll need to listen for the Device Found event, so we will set a handler
    client.set_handler(Some(Box::new(|controller, event| match event {
        ManagementEvent::DeviceFound {
            address,
            address_type,
            flags,
            rssi,
            ..
        } => {
            println!(
                "[{:?}] found device {:?} ({:?})",
                controller, address, address_type
            );
            println!("\tflags: {:?}", flags);
            println!("\trssi: {:?}", rssi);
        }
        _ => (),
    })));

    client
        .start_discovery(
            controller,
            AddressTypeFlag::BREDR | AddressTypeFlag::LEPublic | AddressTypeFlag::LERandom,
        )
        .await?;

    Ok(())
}
