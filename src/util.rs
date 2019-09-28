use std::ffi::CStr;

use bytes::Buf;

pub fn get_string<T>(buf: &mut T, size: usize) -> Option<String> where T: Buf {
    let mut str_bytes = vec![0u8; size];
    buf.copy_to_slice(&mut str_bytes);

    if let Ok(tmp) = CStr::from_bytes_with_nul(&str_bytes) {
        if let Ok(tmp) = tmp.to_str() {
            return Some(tmp.to_owned());
        }
    }

    None
}