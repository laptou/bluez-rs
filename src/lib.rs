//! # Getting Started
//! The most important type in this crate is the [`BlueZClient`](crate::client::BlueZClient), which
//! is used to issue commands to and listen for events from the Bluetooth controller(s). It's
//! fully decked out in `async`/`.await` bling, too.
//!
//! ```no_run
//! # use std::error::Error;
//! # use bluez::client::BlueZClient;
//! # #[async_std::main]
//! # pub async fn main() -> Result<(), Box<dyn Error>> {
//! let mut client = BlueZClient::new().unwrap();
//!
//! let version = client.get_mgmt_version().await?;
//! println!(
//!     "management version: {}.{}",
//!     version.version, version.revision
//! );
//!
//! let controllers = client.get_controller_list().await?;
//!
//! for controller in controllers {
//!     let info = client.get_controller_info(controller).await?;
//!     println!("{:?}", info)
//! }
//!
//! #   Ok(())
//! # }
//! ```
//!
//!
//! Aside from directly issuing commands to the Bluetooth controller and recieving a response,
//! you may want to listen for events or perform processes that span multiple commands. For
//! this to work, you need to supply a callback to your client and call
//! [`BlueZClient::process()`](crate::client::BlueZClient::process). The callback will be
//! called any time that the client processes an event (excluding events that indicate that a
//! command has completed), while `process()` will cause the client to check the underlying
//! socket for new input.
//!
//! ```no_run
//! # use std::error::Error;
//! # use std::time::Duration;
//! # use bluez::client::*;
//! # use bluez::interface::event::Event;
//! # #[async_std::main]
//! # pub async fn main() -> Result<(), Box<dyn Error>> {
//! #    let mut client = BlueZClient::new().unwrap();
//! client.set_handler(Some(Box::new(|controller, event| match event {
//!     Event::DeviceFound {
//!         address,
//!         address_type,
//!         flags,
//!         rssi,
//!         ..
//!     } => {
//!         println!(
//!             "[{:?}] found device {} ({:?})",
//!             controller, address, address_type
//!         );
//!         println!("\tflags: {:?}", flags);
//!         println!("\trssi: {:?}", rssi);
//!     }
//!     _ => (),
//! })));
//!
//! client
//!     .start_discovery(
//!         controller,
//!         AddressTypeFlag::BREDR | AddressTypeFlag::LEPublic | AddressTypeFlag::LERandom,
//!     )
//!     .await?;
//!
//! for _ in 0..5000 {
//!     // don't block if there's no data, just keep looping and sleeping
//!     client.process(false).await?;
//!     std::thread::sleep(Duration::from_millis(50));
//! }
//! #  Ok(())
//! # }
//! ```
//!
//! # Pitfalls
//! Commands that just query information, such as
//! [`BlueZClient::get_controller_info`](crate::client::BlueZClient::get_controller_info),
//! will usually work. However, commands that try to change any settings, such as
//! [`BlueZClient::set_powered`](crate::client::BlueZClient::set_powered) will fail with
//! 'permission denied' errors if your process does not have the `CAP_NET_RAW` capability.

#[macro_use]
extern crate num_derive;
#[macro_use]
extern crate thiserror;

pub use address::Address;
pub use result::{Error, Result};

pub mod client;
pub mod interface;
pub mod result;
pub mod socket;

mod address;
mod util;
