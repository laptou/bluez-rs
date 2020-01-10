use std::ffi::CString;

use bytes::Buf;
use enumflags2::BitFlags;
use enumflags2::_internal::RawBitFlags;
use num_traits::FromPrimitive;

use crate::Address;

pub(crate) trait BufExt2: Buf {
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

    fn get_bool(&mut self) -> bool {
        self.get_u8() != 0
    }

    fn get_primitive_u8<T: FromPrimitive>(&mut self) -> T {
        FromPrimitive::from_u8(self.get_u8()).unwrap()
    }

    fn get_primitive_u16_le<T: FromPrimitive>(&mut self) -> T {
        FromPrimitive::from_u16(self.get_u16_le()).unwrap()
    }

    fn get_flags_u8<T: RawBitFlags<Type = u8>>(&mut self) -> BitFlags<T> {
        BitFlags::from_bits_truncate(self.get_u8())
    }

    fn get_flags_u16_le<T: RawBitFlags<Type = u16>>(&mut self) -> BitFlags<T> {
        BitFlags::from_bits_truncate(self.get_u16_le())
    }

    fn get_flags_u32_le<T: RawBitFlags<Type = u32>>(&mut self) -> BitFlags<T> {
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
}

impl<T: Buf> BufExt2 for T {}
