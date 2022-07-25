//! # Management
//! 
//! For managing the Bluetooth controllers on your device (pairing, discovery,
//! broadcasting, etc.), you can use the management API. This is contained
//! inside of the [`management`] module, where the central type is
//! [`management::ManagementStream`].
//! 
//! # Communication
//! 
//! For communicating with other bluetooth devices, you have a couple of
//! options. You can use L2CAP streams or RFCOMM streams, both of which are
//! exposed through [`BluetoothStream`](crate::communication::BluetoothStream).
//! 
//! This library also contains an implementation of Service Discovery Protocol
//! (SDP) which operates over L2CAP and is availabile in the
//! [`communication::discovery`](crate::communication::discovery) module.
//!
//! # Permissions
//! Commands that just query information, such as
//! [`get_controller_info`](crate::management::get_controller_info),
//! will usually work. However, commands that try to change any settings, such
//! as
//! [`set_powered`](crate::management::set_powered)
//! will fail with 'permission denied' errors if your process does not have the
//! `CAP_NET_ADMIN` capability.

#[macro_use]
extern crate num_derive;
#[macro_use]
extern crate thiserror;

pub use address::*;

pub mod communication;
pub mod management;

mod address;
mod util;
