pub mod socket;
pub mod interface;

#[derive(Debug, Clone, Copy, Fail)]
pub enum ManagementError {
    #[fail(display = "The socket has not been opened yet.")]
    SocketNotOpen,
    #[fail(display = "Unknown error.")]
    Unknown,
    #[fail(display = "Unknown opcode: {:x}.", cmd)]
    UnknownCommand { cmd: u16 },
    #[fail(display = "Unknown command status: {:x}.", status)]
    UnknownStatus { status: u8 },
}
