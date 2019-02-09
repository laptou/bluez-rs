#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate num_derive;

use std::fmt::{Display, Formatter};

mod bt;
mod util;
pub mod mgmt;

#[derive(Debug, Copy, Clone)]
pub struct Address
{
    bytes: [u8; 6]
}

impl Address
{
    fn from_bdaddr(addr: *const bt::bdaddr_t) -> Address
    {
        Address::from(unsafe { (*addr).b })
    }

    fn to_bdaddr(&self) -> bt::bdaddr_t
    {
        bt::bdaddr_t { b: self.bytes }
    }

    pub fn from_slice(bytes: &[u8]) -> Address {
        if bytes.len() != 6 {
            panic!("bluetooth address is 6 bytes");
        }

        let mut arr = [0u8; 6];
        arr.copy_from_slice(bytes);
        Address {
            bytes: arr
        }
    }
}

impl From<[u8; 6]> for Address
{
    fn from(bytes: [u8; 6]) -> Self {
        return Address { bytes };
    }
}

impl From<bt::bdaddr_t> for Address
{
    fn from(addr: bt::bdaddr_t) -> Self {
        return Address { bytes: addr.b };
    }
}

impl Into<bt::bdaddr_t> for Address
{
    fn into(self) -> bt::bdaddr_t {
        return bt::bdaddr_t { b: self.bytes };
    }
}

impl Display for Address
{
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
               self.bytes[5],
               self.bytes[4],
               self.bytes[3],
               self.bytes[2],
               self.bytes[1],
               self.bytes[0])
    }
}