//! This example powers on the first available controller
//! and then starts searching for devices.
//!
//! Copyright (c) 2020 Ibiyemi Abiodun

extern crate bluez;

use std::time::Duration;

use anyhow::{bail, Context};
use bluez::management::*;
use bluez::management::interface::*;
use tokio::time::sleep;

#[tokio::main(flavor = "current_thread")]
pub async fn main() -> std::result::Result<(), anyhow::Error> {
    let mut client = ManagementClient::new().unwrap();

    let controllers = client.get_controller_list().await?;

    let mut active_controller = None;

    for controller in controllers {
        if let Ok(info) = client.get_controller_info(controller).await {
            if info.supported_settings.contains(ControllerSetting::Powered) {
                active_controller = Some((controller, info));
                break;
            } else {
                bail!("controller is not powered");
            }
        }
    }

    let (controller, info) = match active_controller {
        Some(active_controller) => active_controller,
        None => bail!("no available bluetooth controllers"),
    };

    println!("using controller {}", controller);

    if !info.current_settings.contains(ControllerSetting::Powered) {
        println!("powering on bluetooth controller {}", controller);
        client
            .set_powered(controller, true)
            .await
            .context("powering on bluetooth controlled failed")?;
    }

    // scan for some devices
    // to do this we'll need to listen for the Device Found event

    client
        .start_discovery(controller, AddressTypeFlag::BREDR.into())
        .await
        .context("starting discovery failed")?;

    // just wait for discovery forever
    loop {
        // process() blocks until there is a response to be had
        let response = client.process().await.context("processing events failed")?;

        match response.event {
            Event::DeviceFound {
                address,
                address_type,
                flags,
                rssi,
                ..
            } => {
                println!(
                    "[{}] found device {} ({:?})",
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
    }
}
