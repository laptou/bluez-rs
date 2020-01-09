use bytes::Bytes;
use std::ffi::CString;

pub(crate) fn bytes_to_c_str(bytes: Bytes) -> CString {
    let iterator = bytes.into_iter();
    let bytes = iterator.take_while(|&i| i != 0).collect();
    return unsafe { CString::from_vec_unchecked(bytes) };
}