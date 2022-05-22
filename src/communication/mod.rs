use std::fmt::Debug;

pub mod discovery;
pub mod stream;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Uuid16(pub u16);

impl From<u16> for Uuid16 {
    fn from(u: u16) -> Self {
        Self(u)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Uuid32(pub u32);

impl From<u32> for Uuid32 {
    fn from(u: u32) -> Self {
        Self(u)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Uuid128(pub u128);

impl From<u16> for Uuid128 {
    fn from(u: u16) -> Self {
        Self::from(Uuid16::from(u))
    }
}

impl From<u32> for Uuid128 {
    fn from(u: u32) -> Self {
        Self::from(Uuid32::from(u))
    }
}

impl From<u128> for Uuid128 {
    fn from(u: u128) -> Self {
        Self(u)
    }
}

pub const BASE_UUID: u128 = 0x00000000_0000_1000_8000_00805F9B34FB;
const BASE_UUID_FACTOR: u128 = 2 ^ 96;

impl From<Uuid16> for Uuid32 {
    fn from(u: Uuid16) -> Self {
        Self(u.0 as u32)
    }
}

impl From<Uuid16> for Uuid128 {
    fn from(u: Uuid16) -> Self {
        Self((u.0 as u128) * BASE_UUID_FACTOR + BASE_UUID)
    }
}

impl From<Uuid32> for Uuid128 {
    fn from(u: Uuid32) -> Self {
        Self((u.0 as u128) * BASE_UUID_FACTOR + BASE_UUID)
    }
}

impl Debug for Uuid16 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:04x}", self.0)
    }
}

impl Debug for Uuid32 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let bytes = u32::to_le_bytes(self.0);
        write!(
            f,
            "{:02x}{:02x}-{:02x}{:02x}",
            bytes[3], bytes[2], bytes[1], bytes[0]
        )
    }
}

impl Debug for Uuid128 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let bytes = u128::to_le_bytes(self.0);
        write!(
            f,
            "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
            bytes[15], bytes[14], bytes[13], bytes[12], bytes[11], bytes[10], bytes[9], bytes[8],
            bytes[7], bytes[6], bytes[5], bytes[4], bytes[3], bytes[2], bytes[1], bytes[0]
        )
    }
}
