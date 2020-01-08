use thiserror::Error;

pub use result::{ManagementError, Result};

use crate::mgmt::interface::command::{ManagementCommand, ManagementCommandStatus};

mod result;
pub mod client;
pub mod interface;
pub mod socket;

