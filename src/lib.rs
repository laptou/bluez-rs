#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate num_derive;
#[macro_use]
extern crate thiserror;

use std::fmt::{Display, Formatter};

pub mod mgmt;
mod util;

#[derive(Debug, Copy, Clone)]
pub struct Address {
    bytes: [u8; 6],
}

impl Address {
    pub fn from_slice(bytes: &[u8]) -> Address {
        if bytes.len() != 6 {
            panic!("bluetooth address is 6 bytes");
        }

        let mut arr = [0u8; 6];
        arr.copy_from_slice(bytes);
        Address { bytes: arr }
    }
}

impl From<[u8; 6]> for Address {
    fn from(bytes: [u8; 6]) -> Self {
        return Address { bytes };
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
