extern crate bluetooth_rs;

use bluetooth_rs::mgmt;
use bluetooth_rs::mgmt::interface;

pub fn main() -> Result<(), failure::Error> {
    let mut socket = mgmt::socket::ManagementSocket::new()?;
    socket.open()?;
    // Rust version of https://people.csail.mit.edu/albert/bluez-intro/c404.html#bzi-choosing
    let version = interface::get_version(&socket, 5000)?;

    println!("version {}.{}", version.version, version.revision);

    let controllers = interface::get_controllers(&socket, 5000)?;

    for controller in controllers {
        println!("found controller {}", controller);

        let info = interface::get_controller_info(&socket, controller, 5000)?;

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

        if info
            .current_settings
            .contains(interface::ControllerSettings::Connectable)
        {
            println!("connectability already enabled")
        } else {
            println!("enabling connectability");
            let settings = interface::set_connectable(&socket, controller, true, 5000)?;
            println!("new settings: {}", settings);
        }

        println!("enabling discoverability");
        let settings = interface::set_discoverable(
            &socket,
            controller,
            interface::Discoverability::General,
            60,
            -1,
        )?;
        println!("new settings: {}", settings);

        println!("changing name");
        let name = interface::set_name(
            &socket,
            controller,
            "\u{1F171}olvo Stereo",
            Some("VSTEREO"),
            5000,
        )?;
        println!("new name: {:?}", name)
    }

    Ok(())
}
