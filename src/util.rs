use std::ffi::CString;
use std::hash::Hash;
use std::os::unix::prelude::OsStringExt;
use std::{collections::HashMap, ffi::OsString};

use bytes::Buf;
use enumflags2::{BitFlag, BitFlags};
use num_traits::FromPrimitive;

use crate::Address;

pub(crate) trait BufExtBlueZ: Buf {
    fn get_address(&mut self) -> Address {
        let mut arr = [0u8; 6];
        self.copy_to_slice(&mut arr[..]);
        Address::from(arr)
    }

    fn get_u8x16(&mut self) -> [u8; 16] {
        let mut arr = [0u8; 16];
        self.copy_to_slice(&mut arr[..]);
        arr
    }

    fn get_vec_u8(&mut self, len: usize) -> Vec<u8> {
        let mut ret = vec![0; len];
        self.copy_to_slice(ret.as_mut_slice());
        ret
    }

    fn get_bool(&mut self) -> bool {
        self.get_u8() != 0
    }

    fn get_primitive_u8<T: FromPrimitive>(&mut self) -> T {
        FromPrimitive::from_u8(self.get_u8()).unwrap()
    }

    fn get_primitive_u16_le<T: FromPrimitive>(&mut self) -> T {
        FromPrimitive::from_u16(self.get_u16_le()).unwrap()
    }

    fn get_flags_u8<T: BitFlag<Numeric = u8>>(&mut self) -> BitFlags<T> {
        BitFlags::<T, u8>::from_bits_truncate(self.get_u8())
    }

    fn get_flags_u16_le<T: BitFlag<Numeric = u16>>(&mut self) -> BitFlags<T> {
        BitFlags::from_bits_truncate(self.get_u16_le())
    }

    fn get_flags_u32_le<T: BitFlag<Numeric = u32>>(&mut self) -> BitFlags<T> {
        BitFlags::from_bits_truncate(self.get_u32_le())
    }

    fn get_c_string(&mut self) -> CString {
        let mut bytes = vec![];
        let mut current = self.get_u8();
        while current != 0 && self.has_remaining() {
            bytes.push(current);
            current = self.get_u8();
        }
        return unsafe { CString::from_vec_unchecked(bytes) };
    }

    /// Parses a list of Type/Length/Value entries into a map keyed by type
    ///
    /// This parses a list of mgmt_tlv entries (as defined in mgmt.h) and converts them
    /// into a map of Type => Vec<u8>.
    ///
    /// # Bytes layout
    ///
    /// The layout as described in the mgmt-api documentation is:
    /// ```plain
    ///   Parameter1 {
    ///       Parameter_Type (2 Octet)
    ///       Value_Length (1 Octet)
    ///       Value (0-255 Octets)
    ///   }
    ///   Parameter2 { }
    ///   ...
    /// ```
    ///
    fn get_tlv_map<T: FromPrimitive + Eq + Hash>(&mut self) -> HashMap<T, Vec<u8>> {
        let mut parameters = HashMap::new();
        while self.has_remaining() {
            let parameter_type: T = self.get_primitive_u16_le();
            let value_size = self.get_u8() as usize;
            parameters.insert(parameter_type, self.get_vec_u8(value_size));
        }
        parameters
    }
}

impl<T: Buf> BufExtBlueZ for T {}

pub(crate) fn check_error(value: libc::c_int) -> Result<libc::c_int, std::io::Error> {
    if value < 0 {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(value)
    }
}
