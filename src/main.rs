extern crate bluetooth_rs;

use bluetooth_rs::hci::device::{Device, Socket};
use bluetooth_rs::hci::inquiry;

pub fn main() -> Result<(), failure::Error> {
    // Rust version of https://people.csail.mit.edu/albert/bluez-intro/c404.html#bzi-choosing
    let device = Device::default()?;
    let socket = device.open()?;
    let nearby = inquiry::inquire(&device, 10, 255, true)?;

    for (i, nearby) in nearby.iter().enumerate() {
        let name = socket.get_friendly_name(nearby.address, 5000)?;
        println!(
            "#{}:\tname: {} address: {} service classes: {:?} device class: {:?}",
            i, name, nearby.address, nearby.service_classes, nearby.device_class
        );
    }

    Ok(())
}
