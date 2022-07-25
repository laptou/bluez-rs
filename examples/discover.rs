//! This example powers on the first available controller
//! and then starts searching for devices.
//!
//! Copyright (c) 2022 Ibiyemi Abiodun

extern crate bluez;

use anyhow::{bail, Context};
use bluez::management::interface::*;
use bluez::management::*;

#[tokio::main(flavor = "current_thread")]
pub async fn main() -> std::result::Result<(), anyhow::Error> {
    let mut mgmt = ManagementStream::open().context("failed to connect to mgmt api")?;

    let controllers = get_controller_list(&mut mgmt, None).await?;

    let mut active_controller = None;

    for controller in controllers {
        if let Ok(info) = get_controller_info(&mut mgmt, controller, None).await {
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
        set_powered(&mut mgmt, controller, true, None)
            .await
            .context("powering on bluetooth controlled failed")?;
    }

    // stop discovery if it is active
    let _ = stop_discovery(&mut mgmt, controller, AddressTypeFlag::BREDR.into(), None).await;

    // scan for some devices
    // to do this we'll need to listen for the Device Found event

    start_discovery(&mut mgmt, controller, AddressTypeFlag::BREDR.into(), None)
        .await
        .context("starting discovery failed")?;

    // just wait for discovery forever
    loop {
        let response = mgmt.receive().await.context("receiving events failed")?;

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
                    start_discovery(
                        &mut mgmt,
                        controller,
                        AddressTypeFlag::BREDR
                            | AddressTypeFlag::LEPublic
                            | AddressTypeFlag::LERandom,
                        None,
                    )
                    .await?;
                }
            }
            _ => (),
        }
    }
}
