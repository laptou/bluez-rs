extern crate bluez;

use std::error::Error;
use std::time::Duration;

use async_std::task::block_on;

use bluez::client::*;
use bluez::interface::controller::*;
use bluez::interface::event::Event;

#[async_std::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
    let mut client = BlueZClient::new().unwrap();

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
        println!("\t\taddress: {:?}", info.address);
        println!("\t\tsupported settings: {:?}", info.supported_settings);
        println!("\t\tcurrent settings: {:?}", info.current_settings);
        println!("\t\tmanufacturer: 0x{:04x}", info.manufacturer);
        println!("\t\tbluetooth version: 0x{:02x}", info.bluetooth_version);
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
        Event::DeviceFound {
            address,
            address_type,
            flags,
            rssi,
            ..
        } => {
            println!(
                "[{:?}] found device {} ({:?})",
                controller, address, address_type
            );
            println!("\tflags: {:?}", flags);
            println!("\trssi: {:?}", rssi);
        }
        Event::Discovering {
            discovering,
            address_type,
        } => println!("discovering: {} {:?}", discovering, address_type),
        _ => (),
    })));

    client
        .start_discovery(
            controller,
            AddressTypeFlag::BREDR | AddressTypeFlag::LEPublic | AddressTypeFlag::LERandom,
        )
        .await?;

    for _ in 0..5000 {
        // don't block if there's no data, just keep looping and sleeping
        client.process(false).await?;
        std::thread::sleep(Duration::from_millis(50));
    }

    Ok(())
}
