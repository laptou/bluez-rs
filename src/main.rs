extern crate bluez;

use std::error::Error;

use async_std::task;

use bluez::mgmt;
use bluez::mgmt::interface;
use bluez::mgmt::interface::ManagementCommand::PinCodeReply;

pub fn main() -> Result<(), Box<dyn Error>> {
    task::block_on(main_async())
}

pub async fn main_async() -> Result<(), Box<dyn Error>> {
    let mut client = mgmt::client::ManagementClient::new();

    // Rust version of https://people.csail.mit.edu/albert/bluez-intro/c404.html#bzi-choosing
    let version = client.get_mgmt_version().await?;
    println!("management version: {}.{}", version.version, version.revision);

    let controllers = client.get_controller_list().await?;
    println!();
    println!("available controllers:");

    for controller in controllers {
        println!("\t{:?}", controller);
        let info = client.get_controller_info(controller).await?;
        println!("\t\tname: {:?}", info.name);
        println!("\t\tshort name: {:?}", info.short_name);
        println!("\t\tsupported settings: {}", info.supported_settings);
        println!("\t\tcurrent settings: {}", info.current_settings);
        println!("\t\tmanufacturer: {:?}", info.manufacturer);
        println!("\t\tbluetooth version: {:?}", info.bluetooth_version);
        println!("\t\tclass of device: {:?}", info.class_of_device);
    }

    Ok(())
}
