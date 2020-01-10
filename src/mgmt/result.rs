use crate::mgmt::interface::{ManagementCommand, ManagementCommandStatus};

pub type Result<T> = std::result::Result<T, ManagementError>;

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
        max_len
    )]
    NameTooLong { name: String, max_len: u32 },
    #[error("A string was supplied that contained a null byte: {}", source)]
    NullByte {
        #[source]
        source: ::std::ffi::NulError,
    },
    #[error("The pin code is too long; the maximum length is {} bytes.", max_len)]
    PinCodeTooLong { max_len: u32 },
}

impl From<std::io::Error> for ManagementError {
    fn from(err: std::io::Error) -> Self {
        ManagementError::IO { source: err }
    }
}

impl From<std::ffi::NulError> for ManagementError {
    fn from(err: std::ffi::NulError) -> Self {
        ManagementError::NullByte { source: err }
    }
}
