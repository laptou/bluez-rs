//! # Getting Started
//! The most important type in this crate is the [ManagementClient](crate::client::ManagementClient).
//! It is the type you will use to interface with the Bluetooth controller.
//!
//! # Pitfalls
//! Commands that just query information, such as [crate::client::ManagementClient::get_controller_info],
//! will usually work. However, commands that try to change any settings, such as
//! [crate::client::ManagementClient::set_powered] will fail with [crate::interface

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