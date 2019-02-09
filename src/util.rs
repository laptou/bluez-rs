use std::ffi::CStr;

#[inline]
pub fn read_u16_le(buf: &Vec<u8>, offset: usize) -> u16 {
    let mut c = [0u8; 2];
    c.copy_from_slice(&buf[offset..offset + 2]);
    u16::from_le_bytes(c)
}

#[inline]
pub fn read_u32_le(buf: &Vec<u8>, offset: usize) -> u32 {
    let mut c = [0u8; 4];
    c.copy_from_slice(&buf[offset..offset + 4]);
    u32::from_le_bytes(c)
}

pub fn read_str(buf: &Vec<u8>, offset: usize, max_len: usize) -> Option<String> {
    // measure length first
    // b/c most of the buffer is likely full of zeroes
    let len = &buf[offset..offset + max_len]
        .iter()
        .take_while(|&&b| b != 0u8)
        .count() + 1;

    if len == 0 {
        return Some("".to_owned());
    }

    CStr::from_bytes_with_nul(&buf[offset..offset + len])
        .ok()
        .and_then(|s| s.to_owned().into_string().ok())
}

pub trait LE<T> {
    fn to_le_bytes(self) -> T;
    fn from_le_bytes(_: T) -> Self;
}

impl LE<[u8; 2]> for u16 {
    fn to_le_bytes(self) -> [u8; 2] {
        unsafe { ::std::mem::transmute(self.to_le()) }
    }

    fn from_le_bytes(bytes: [u8; 2]) -> Self {
        let n: u16 = unsafe { ::std::mem::transmute(bytes) };

        if cfg!(target_endian = "big") {
            n.to_le() // flip endianness
        } else {
            n
        }
    }
}

impl LE<[u8; 4]> for u32 {
    fn to_le_bytes(self) -> [u8; 4] {
        unsafe { ::std::mem::transmute(self.to_le()) }
    }

    fn from_le_bytes(bytes: [u8; 4]) -> Self {
        let n: u32 = unsafe { ::std::mem::transmute(bytes) };

        if cfg!(target_endian = "big") {
            n.to_le() // flip endianness
        } else {
            n
        }
    }
}

#[cfg(test)]
mod tests {
    use super::LE;

    #[test]
    fn endian() {
        assert_eq!(0xFA0032u32.to_le_bytes(), [0x32, 0x00, 0xFA, 0x00]);
        assert_eq!(0xFA32u16.to_le_bytes(), [0x32, 0xFA]);
        assert_eq!(u32::from_le_bytes([0x32, 0x00, 0xFA, 0x00]), 0xFA0032u32);
        assert_eq!(u16::from_le_bytes([0x32, 0xFA]), 0xFA32u16);
    }
}