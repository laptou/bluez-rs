use crate::mgmt::interface::command::{ManagementCommand, ManagementCommandStatus};

pub mod interface;
pub mod socket;
pub mod client;

#[derive(Debug, Fail)]
pub enum ManagementError {
    #[fail(display = "Unknown error.")]
    Unknown,
    #[fail(display = "IO error: {:?}.", err)]
    IO { err: ::std::io::Error },
    #[fail(display = "Command {:?} returned {:?}.", status, opcode)]
    CommandError {
        opcode: ManagementCommand,
        status: ManagementCommandStatus,
    },
    #[fail(display = "Unknown opcode: {:x}.", opcode)]
    UnknownOpcode { opcode: u16 },
    #[fail(display = "Unknown command status: {:x}.", status)]
    UnknownStatus { status: u8 },
    #[fail(display = "Timed out.")]
    TimedOut,
    #[fail(display = "The socket received invalid data.")]
    InvalidData,
    #[fail(
    display = "The name {} is too long; the maximum length is {} bytes.",
    name, maxlen
    )]
    NameTooLong { name: String, maxlen: u32 },
    #[fail(
    display = "The pin code is too long; the maximum length is {} bytes.",
    maxlen
    )]
    PinCodeTooLong { maxlen: u32 },
}

impl From<std::io::Error> for ManagementError {
    fn from(err: std::io::Error) -> Self {
        ManagementError::IO { err }
    }
}