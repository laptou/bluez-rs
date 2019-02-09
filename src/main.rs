extern crate bluetooth_rs;

use bluetooth_rs::mgmt;

pub fn main() -> Result<(), failure::Error> {
    let mut socket = mgmt::socket::ManagementSocket::new()?;
    socket.open()?;
    // Rust version of https://people.csail.mit.edu/albert/bluez-intro/c404.html#bzi-choosing
    let version = mgmt::interface::get_version(&socket, 5000)?;

    println!("version {}.{}", version.version, version.revision);

    let controllers = mgmt::interface::get_controllers(&socket, 5000)?;

    for controller in controllers {
        println!("found controller: {:x}", controller);

        let info = mgmt::interface::get_controller_info(&socket, controller, 5000)?;

        println!("\taddress: {}", info.address);

        if let Some(short_name) = info.short_name {
            println!("\tshort name: {}", short_name);
        } else {
            println!("\tshort name: (none)");
        }

        if let Some(name) = info.name {
            println!("\tname: {}", name);
        } else {
            println!("\tname: (none)");
        }

        println!("\tsupported settings: {}", info.supported_settings);
        println!("\tcurrent settings: {}", info.current_settings);
        println!();
    }

    Ok(())
}
