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