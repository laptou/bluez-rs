//! This example powers on the first available controller
//! and then starts searching for devices.
//!
//! Copyright (c) 2020 Ibiyemi Abiodun

extern crate bluez;

use std::error::Error;
use std::time::Duration;

use async_std::task::{block_on, sleep};

use bluez::client::*;
use bluez::interface::controller::*;
use bluez::interface::event::Event;

#[async_std::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
    let mut client = ManagementClient::new().unwrap();

    let controllers = client.get_controller_list().await?;

    // find the first controller we can power on
    let (controller, info) = controllers
        .into_iter()
        .filter_map(|controller| {
            let info = block_on(client.get_controller_info(controller)).ok()?;

            if info.supported_settings.contains(ControllerSetting::Powered) {
                Some((controller, info))
            } else {
                None
            }
        })
        .nth(0)
        .expect("no usable controllers found");

    if !info.current_settings.contains(ControllerSetting::Powered) {
        println!("powering on bluetooth controller {}", controller);
        client.set_powered(controller, true).await?;
    }

    // scan for some devices
    // to do this we'll need to listen for the Device Found event

    client
        .start_discovery(
            controller,
            AddressTypeFlag::BREDR | AddressTypeFlag::LEPublic | AddressTypeFlag::LERandom,
        )
        .await?;

    // just wait for discovery forever
    loop {
        // process() blocks until there is a response to be had
        let response = client.process().await?;

        match response.event {
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
            } => {
                println!("discovering: {} {:?}", discovering, address_type);

                // if discovery ended, turn it back on
                if !discovering {
                    client
                        .start_discovery(
                            controller,
                            AddressTypeFlag::BREDR
                                | AddressTypeFlag::LEPublic
                                | AddressTypeFlag::LERandom,
                        )
                        .await?;
                }
            }
            _ => (),
        }

        sleep(Duration::from_millis(50)).await;
    }
}
