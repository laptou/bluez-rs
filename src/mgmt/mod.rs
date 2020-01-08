use thiserror::Error;

use crate::mgmt::interface::command::{ManagementCommand, ManagementCommandStatus};

pub mod client;
pub mod interface;
pub mod socket;

#[derive(Error, Debug)]
pub enum ManagementError {
    #[error("Unknown error.")]
    Unknown,
    #[error("IO error: {:?}.", source)]
    IO {
        #[source]
        source: ::std::io::Error,
    },
    #[error("Command {:?} returned {:?}.", status, opcode)]
    CommandError {
        opcode: ManagementCommand,
        status: ManagementCommandStatus,
    },
    #[error("Unknown opcode: {:x}.", opcode)]
    UnknownOpcode { opcode: u16 },
    #[error("Unknown command status: {:x}.", status)]
    UnknownStatus { status: u8 },
    #[error("Timed out.")]
    TimedOut,
    #[error("The socket received invalid data.")]
    InvalidData,
    #[error(
    "The name {} is too long; the maximum length is {} bytes.",
    name,
    maxlen
    )]
    NameTooLong { name: String, maxlen: u32 },
    #[error("The pin code is too long; the maximum length is {} bytes.", maxlen)]
    PinCodeTooLong { maxlen: u32 },
}

impl From<std::io::Error> for ManagementError {
    fn from(err: std::io::Error) -> Self {
        ManagementError::IO { source: err }
    }
}
