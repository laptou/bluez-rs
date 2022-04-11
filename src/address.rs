use std::fmt::{Display, Formatter};

use bytes::Buf;

#[repr(C, packed)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Default)]
pub struct Address {
    bytes: [u8; 6],
}

impl Address {
    pub const fn new(bytes: [u8; 6]) -> Address {
        Address { bytes }
    }

    pub fn from_buf<B: Buf>(buf: &mut B) -> Address {
        if buf.remaining() < 6 {
            panic!("bluetooth address is 6 bytes");
        }

        let mut arr = [0u8; 6];
        buf.copy_to_slice(&mut arr[..]);
        Address::new(arr)
    }

    pub fn from_slice(bytes: &[u8]) -> Address {
        if bytes.len() != 6 {
            panic!("bluetooth address is 6 bytes");
        }

        let mut arr = [0u8; 6];
        arr.copy_from_slice(bytes);
        Address::new(arr)
    }

    pub const fn zero() -> Address {
        Address { bytes: [0u8; 6] }
    }
}

impl From<[u8; 6]> for Address {
    fn from(bytes: [u8; 6]) -> Self {
        return Address { bytes };
    }
}

impl From<Address> for [u8; 6] {
    fn from(val: Address) -> Self {
        val.bytes
    }
}

impl AsRef<[u8]> for Address {
    fn as_ref(&self) -> &[u8] {
        &self.bytes
    }
}

impl Display for Address {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        write!(
            f,
            "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
            self.bytes[5],
            self.bytes[4],
            self.bytes[3],
            self.bytes[2],
            self.bytes[1],
            self.bytes[0]
        )
    }
}
