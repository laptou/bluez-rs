use std::{
    fmt::{Display, Formatter},
    str::FromStr,
};

use bytes::Buf;

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
        Address { bytes }
    }
}

impl From<Address> for [u8; 6] {
    fn from(val: Address) -> Self {
        val.bytes
    }
}

impl Into<bluez_sys::bdaddr_t> for Address {
    fn into(self) -> bluez_sys::bdaddr_t {
        bluez_sys::bdaddr_t { b: self.bytes }
    }
}

impl From<bluez_sys::bdaddr_t> for Address {
    fn from(bdaddr: bluez_sys::bdaddr_t) -> Self {
        Address { bytes: bdaddr.b }
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

impl FromStr for Address {
    type Err = AddressParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut it = s
            .split(':')
            .map(|octet| u8::from_str_radix(octet, 16))
            .rev();

        let bytes = [
            it.next()
                .ok_or(AddressParseError::NotEnoughOctets)?
                .or(Err(AddressParseError::InvalidOctet))?,
            it.next()
                .ok_or(AddressParseError::NotEnoughOctets)?
                .or(Err(AddressParseError::InvalidOctet))?,
            it.next()
                .ok_or(AddressParseError::NotEnoughOctets)?
                .or(Err(AddressParseError::InvalidOctet))?,
            it.next()
                .ok_or(AddressParseError::NotEnoughOctets)?
                .or(Err(AddressParseError::InvalidOctet))?,
            it.next()
                .ok_or(AddressParseError::NotEnoughOctets)?
                .or(Err(AddressParseError::InvalidOctet))?,
            it.next()
                .ok_or(AddressParseError::NotEnoughOctets)?
                .or(Err(AddressParseError::InvalidOctet))?,
        ];

        Ok(Self { bytes })
    }
}

#[derive(Error, Debug, Clone, Copy)]
pub enum AddressParseError {
    #[error("the string contained an invalid octet")]
    InvalidOctet,
    #[error("the string contained less than six octets")]
    NotEnoughOctets,
    #[error("the string contained more than six octets")]
    TooManyOctets,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, FromPrimitive)]
pub enum AddressType {
    BREDR = 0,
    LEPublic = 1,
    LERandom = 2,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, FromPrimitive, ToPrimitive)]
pub enum Protocol {
    L2CAP = bluez_sys::BTPROTO_L2CAP,
    HCI = bluez_sys::BTPROTO_HCI,
    RFCOMM = bluez_sys::BTPROTO_RFCOMM,
}
