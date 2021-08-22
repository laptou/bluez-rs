use crate::management::interface::{Command, CommandStatus};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Unknown error.")]
    Unknown,
    #[error("No data was available to be read.")]
    NoData,
    #[error("IO error: {:?}.", source)]
    IO {
        #[source]
        source: ::std::io::Error,
    },
    #[error("Command {:?} returned {:?}.", opcode, status)]
    CommandError {
        opcode: Command,
        status: CommandStatus,
    },
    #[error("Unknown opcode: {:x}.", opcode)]
    UnknownOpcode { opcode: u16 },
    #[error("Unknown command status: {:x}.", status)]
    UnknownStatus { status: u8 },
    #[error("Unknown event code: {:x}.", evt_code)]
    UnknownEventCode { evt_code: u16 },
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

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::IO { source: err }
    }
}

impl From<std::ffi::NulError> for Error {
    fn from(err: std::ffi::NulError) -> Self {
        Error::NullByte { source: err }
    }
}
