use bytes::Buf;
use num_traits::FromPrimitive;

#[derive(Error, Debug)]
pub enum Error {
    #[error("an i/o error occurred")]
    Io(#[from] std::io::Error),

    #[error("the remote device returned an error: {0:?}")]
    Remote(ErrorCode),

    #[error("the remote device returned invalid data")]
    InvalidResponse,
}

#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromPrimitive, ToPrimitive)]
pub enum ErrorCode {
    UnsupportedSdpVersion = 0x0001,
    InvalidServiceRecordHandle,
    InvalidRequestSyntax,
    InvalidPduSize,
    InvalidContinuationState,
    InsufficientResources,
}

impl<B: Buf> From<&mut B> for ErrorCode {
    fn from(buf: &mut B) -> Self {
        let code = buf.get_u16();
        FromPrimitive::from_u16(code).unwrap()
    }
}
