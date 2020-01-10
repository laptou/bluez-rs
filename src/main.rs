extern crate bluez;

use std::error::Error;

use futures::stream::{self, StreamExt};

use bluez::client::ManagementClient;
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
    let mut controllers_info = stream::iter(controllers)
        .map(async |controller| {
            (
                controller,
                client.get_controller_info(*controller).await.unwrap(),
            );
        })
        .collect::<Vec<(Controller, ControllerInfo)>>()
        .await;

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

    // power it on
    client.set_powered(controller, true);

    // scan for some devices
    // to do this we'll need to listen for the Device Found event, so we will set a handler
    client.set_handler(Some(Box::new(|controller, event| {
        match event {
            ManagementEvent::DeviceFound { address, address_type, flags, rssi, .. } => {
                println!()
            }
            _ => ()
        }
    })));

    Ok(())
}
