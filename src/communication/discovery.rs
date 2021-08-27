use std::{
    borrow::Borrow, ffi::CStr, fmt::Display, iter::FromIterator, marker::PhantomData,
    mem::ManuallyDrop,
};

use enumflags2::{bitflags, BitFlags};

use crate::{socket::BtProto, Address};

use super::stream::BluetoothStream;

#[repr(transparent)]
#[derive(Copy, Clone)]

pub struct SdpUuid(bluetooth_sys::uuid_t);

#[repr(u8)]
pub enum SdpUuidKind {
    Uuid128 = bluetooth_sys::SDP_UUID128 as u8,
    Uuid32 = bluetooth_sys::SDP_UUID32 as u8,
    Uuid16 = bluetooth_sys::SDP_UUID16 as u8,
}

impl SdpUuid {
    pub fn kind(&self) -> SdpUuidKind {
        match self.0.type_ as u32 {
            bluetooth_sys::SDP_UUID128 => SdpUuidKind::Uuid128,
            bluetooth_sys::SDP_UUID32 => SdpUuidKind::Uuid32,
            bluetooth_sys::SDP_UUID16 => SdpUuidKind::Uuid16,
            _ => unreachable!(),
        }
    }
}

impl From<[u8; 16]> for SdpUuid {
    fn from(b: [u8; 16]) -> Self {
        let mut uuid = unsafe { std::mem::zeroed() };
        unsafe {
            bluetooth_sys::sdp_uuid128_create(&mut uuid, &b as *const _ as *const _);
        }
        Self(uuid)
    }
}

impl From<u32> for SdpUuid {
    fn from(b: u32) -> Self {
        let mut uuid = unsafe { std::mem::zeroed() };
        unsafe {
            bluetooth_sys::sdp_uuid32_create(&mut uuid, b);
        }
        Self(uuid)
    }
}

impl From<u16> for SdpUuid {
    fn from(b: u16) -> Self {
        let mut uuid = unsafe { std::mem::zeroed() };
        unsafe {
            bluetooth_sys::sdp_uuid16_create(&mut uuid, b);
        }
        Self(uuid)
    }
}

impl Into<bluetooth_sys::uuid_t> for SdpUuid {
    fn into(self) -> bluetooth_sys::uuid_t {
        self.0
    }
}

impl Display for SdpUuid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut buf = [0; bluetooth_sys::MAX_LEN_UUID_STR as usize];
        let err = unsafe { bluetooth_sys::sdp_uuid2strn(&self.0, &mut buf[0], buf.len() as u64) };

        if err < 0 {
            panic!("printing uuid failed");
        }

        let formatted_uuid = unsafe { CStr::from_ptr(&buf[0]) };

        f.write_str(formatted_uuid.to_string_lossy().borrow())?;

        Ok(())
    }
}

impl std::fmt::Debug for SdpUuid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

#[repr(u32)]
pub enum SdpAttributeSpecification {
    // 16bit individual identifier. They are the actual attribute identifiers in
    // ascending order
    Individual = bluetooth_sys::sdp_attrreq_type_t_SDP_ATTR_REQ_INDIVIDUAL,

    // 32bit identifier range. The high-order 16bits is the start of range the
    // low-order 16bits are the end of range 0x0000 to 0xFFFF gets all
    // attributes
    Range = bluetooth_sys::sdp_attrreq_type_t_SDP_ATTR_REQ_RANGE,
}

#[bitflags]
#[repr(u32)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SdpSessionFlags {
    RetryIfBusy = bluetooth_sys::SDP_RETRY_IF_BUSY,
    WaitOnClose = bluetooth_sys::SDP_WAIT_ON_CLOSE,
    NonBlocking = bluetooth_sys::SDP_NON_BLOCKING,
    LargeMtu = bluetooth_sys::SDP_LARGE_MTU,
}

/// This is a linked list provided by the kernel API.
#[repr(transparent)]
pub struct SdpList<'a, T> {
    inner: *mut bluetooth_sys::sdp_list_t,
    _data: PhantomData<&'a mut T>,
}

impl<'a, T> SdpList<'a, T> {
    pub fn create(item: &'a T) -> Self {
        SdpList {
            inner: unsafe {
                bluetooth_sys::sdp_list_append(std::ptr::null_mut(), item as *const T as *mut _)
            },
            _data: PhantomData,
        }
    }

    pub fn append(&mut self, item: &'a T) {
        self.inner =
            unsafe { bluetooth_sys::sdp_list_append(self.inner, item as *const T as *mut _) };
    }

    pub fn iter(&self) -> SdpListIter<T> {
        SdpListIter {
            current: self.inner,
            _data: &PhantomData,
        }
    }

    pub fn iter_mut(&mut self) -> SdpListIterMut<T> {
        SdpListIterMut {
            current: self.inner,
            _data: &PhantomData,
        }
    }

    pub fn into_iter(self) -> SdpListIntoIter<'a, T> {
        SdpListIntoIter {
            current: self.inner,
            _list: self,
        }
    }
}

pub struct SdpListIter<'a, T> {
    current: *mut bluetooth_sys::sdp_list_t,
    _data: &'a PhantomData<T>,
}

impl<'a, T> Iterator for SdpListIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        match unsafe { self.current.as_ref() } {
            Some(list) => {
                let item = list.data as *mut T;
                let item = unsafe { item.as_ref() };
                self.current = list.next;
                item
            }
            None => None,
        }
    }
}

pub struct SdpListIterMut<'a, T> {
    current: *mut bluetooth_sys::sdp_list_t,
    _data: &'a PhantomData<T>,
}

impl<'a, T> Iterator for SdpListIterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        match unsafe { self.current.as_ref() } {
            Some(list) => {
                let item = list.data as *mut T;
                let item = unsafe { item.as_mut() };
                self.current = list.next;
                item
            }
            None => None,
        }
    }
}

pub struct SdpListIntoIter<'a, T> {
    current: *mut bluetooth_sys::sdp_list_t,
    _list: SdpList<'a, T>,
}

impl<'a, T> Iterator for SdpListIntoIter<'a, T> {
    type Item = *mut T;

    fn next(&mut self) -> Option<Self::Item> {
        match unsafe { self.current.as_ref() } {
            Some(list) => {
                let item = list.data as *mut T;
                self.current = list.next;
                Some(item)
            }
            None => None,
        }
    }
}

impl<'a, T> FromIterator<&'a T> for SdpList<'a, T> {
    fn from_iter<U: IntoIterator<Item = &'a T>>(iter: U) -> Self {
        let iter = iter.into_iter();
        let mut list = SdpList {
            inner: std::ptr::null_mut(),
            _data: PhantomData,
        };

        for item in iter {
            list.append(item);
        }

        list
    }
}

impl<'a, T> Drop for SdpList<'a, T> {
    fn drop(&mut self) {
        unsafe { bluetooth_sys::sdp_list_free(self.inner, None) }
    }
}

#[repr(transparent)]
pub struct SdpRecord<'a>(
    *mut bluetooth_sys::sdp_record_t,
    PhantomData<&'a bluetooth_sys::sdp_record_t>,
);

impl<'a> SdpRecord<'a> {
    pub fn handle(&self) -> u32 {
        unsafe { *self.0 }.handle
    }

    // Do not drop these SdpData instances. They are owned by.
    pub fn get_access_protos(&self) -> std::io::Result<Vec<ManuallyDrop<SdpData<'a>>>> {
        let mut list = std::ptr::null_mut();
        let res = unsafe { bluetooth_sys::sdp_get_access_protos(self.0, &mut list) };
        if res < 0 {
            return Err(std::io::Error::from_raw_os_error(res));
        }

        let list: SdpList<bluetooth_sys::sdp_data_t> = SdpList {
            inner: list,
            _data: PhantomData,
        };

        let list = list
            .into_iter()
            .map(|data| {
                ManuallyDrop::new(SdpData(data, PhantomData))
            })
            .collect();

        Ok(list)
    }
}

impl<'a> Drop for SdpRecord<'a> {
    fn drop(&mut self) {
        unsafe { bluetooth_sys::sdp_record_free(self.0) }
    }
}

#[repr(transparent)]
pub struct SdpData<'a>(
    *mut bluetooth_sys::sdp_data_t,
    PhantomData<&'a bluetooth_sys::sdp_data_t>,
);

#[derive(Debug, Copy, Clone)]
pub enum SdpDataValue<'a> {
    None,
    U8(u8),
    I8(i8),
    BOOL(bool),
    U16(u16),
    I16(i16),
    U32(u32),
    I32(i32),
    I64(i64),
    U64(u64),
    U128(u128),
    I128(i128),
    Uuid(SdpUuid),
    Url(&'a CStr),
    Text(&'a CStr),
    ALT8(&'a [u8]),
    ALT16(&'a [u16]),
    ALT32(&'a [u32]),
    SEQ8(&'a [u8]),
    SEQ16(&'a [u16]),
    SEQ32(&'a [u32]),
}

impl<'a> SdpData<'a> {
    pub fn value(&self) -> SdpDataValue {
        unsafe {
            let inner = &*self.0;

            match inner.dtd as u32 {
                bluetooth_sys::SDP_DATA_NIL => SdpDataValue::None,
                bluetooth_sys::SDP_UINT8 => SdpDataValue::U8(inner.val.uint8),
                bluetooth_sys::SDP_INT8 => SdpDataValue::I8(inner.val.int8),
                bluetooth_sys::SDP_BOOL => SdpDataValue::BOOL(inner.val.int8 != 0),
                bluetooth_sys::SDP_UINT16 => SdpDataValue::U16(inner.val.uint16),
                bluetooth_sys::SDP_INT16 => SdpDataValue::I16(inner.val.int16),
                bluetooth_sys::SDP_UINT32 => SdpDataValue::U32(inner.val.uint32),
                bluetooth_sys::SDP_INT32 => SdpDataValue::I32(inner.val.int32),
                bluetooth_sys::SDP_INT64 => SdpDataValue::I64(inner.val.int64),
                bluetooth_sys::SDP_UINT64 => SdpDataValue::U64(inner.val.uint64),
                bluetooth_sys::SDP_UINT128 => {
                    SdpDataValue::U128(u128::from_le_bytes(inner.val.uint128.data))
                }
                bluetooth_sys::SDP_INT128 => {
                    SdpDataValue::I128(i128::from_le_bytes(inner.val.int128.data))
                }
                bluetooth_sys::SDP_UUID16
                | bluetooth_sys::SDP_UUID32
                | bluetooth_sys::SDP_UUID128 => SdpDataValue::Uuid(SdpUuid(inner.val.uuid)),
                bluetooth_sys::SDP_TEXT_STR8
                | bluetooth_sys::SDP_TEXT_STR16
                | bluetooth_sys::SDP_TEXT_STR32 => {
                    SdpDataValue::Text(CStr::from_ptr(inner.val.str))
                }
                bluetooth_sys::SDP_URL_STR8
                | bluetooth_sys::SDP_URL_STR16
                | bluetooth_sys::SDP_URL_STR32 => SdpDataValue::Url(CStr::from_ptr(inner.val.str)),
                _ => unimplemented!(),
            }
        }
    }
}

impl<'a> Drop for SdpData<'a> {
    fn drop(&mut self) {
        unsafe { bluetooth_sys::sdp_data_free(self.0) }
    }
}

pub struct SdpSession(*mut bluetooth_sys::sdp_session_t);

impl SdpSession {
    pub fn connect(
        source_addr: Address,
        target_addr: Address,
        flags: BitFlags<SdpSessionFlags>,
    ) -> std::io::Result<SdpSession> {
        let source_addr = source_addr.into();
        let target_addr = target_addr.into();
        let session =
            unsafe { bluetooth_sys::sdp_connect(&source_addr, &target_addr, flags.bits()) };

        if session.is_null() {
            return Err(std::io::Error::last_os_error());
        }

        return Ok(Self(session));
    }

    pub fn search_req(
        &self,
        search_list: &[SdpUuid],
        max_rec_num: u16,
    ) -> std::io::Result<Vec<u16>> {
        let search_list: SdpList<SdpUuid> = search_list.into_iter().collect();
        let mut rsp_list: *mut bluetooth_sys::sdp_list_t = std::ptr::null_mut();

        self.check_error(unsafe {
            bluetooth_sys::sdp_service_search_req(
                self.0,
                search_list.inner,
                max_rec_num,
                &mut rsp_list,
            )
        })?;

        let rsp_list = SdpList {
            inner: rsp_list,
            _data: PhantomData,
        };

        let items = rsp_list.iter().map(|record| *record).collect();

        Ok(items)
    }

    pub fn search_attr_req<'a>(
        &self,
        search_list: &[SdpUuid],
        req_type: SdpAttributeSpecification,
        attr_list: &[u32],
    ) -> std::io::Result<Vec<SdpRecord<'a>>> {
        let search_list: SdpList<SdpUuid> = search_list.into_iter().collect();
        let attr_list: SdpList<u32> = attr_list.into_iter().collect();
        let mut rsp_list: *mut bluetooth_sys::sdp_list_t = std::ptr::null_mut();

        self.check_error(unsafe {
            bluetooth_sys::sdp_service_search_attr_req(
                self.0,
                search_list.inner,
                req_type as u32,
                attr_list.inner,
                &mut rsp_list,
            )
        })?;

        let rsp_list = SdpList {
            inner: rsp_list,
            _data: PhantomData,
        };

        let result = rsp_list
            .into_iter()
            .map(|item| SdpRecord(item, PhantomData))
            .collect();

        Ok(result)
    }

    fn check_error(&self, retval: i32) -> std::io::Result<()> {
        if retval < 0 {
            let err = unsafe { bluetooth_sys::sdp_get_error(self.0) };
            return Err(std::io::Error::from_raw_os_error(err));
        }

        Ok(())
    }
}

impl Drop for SdpSession {
    fn drop(&mut self) {
        unsafe {
            bluetooth_sys::sdp_close(self.0);
        }
    }
}

impl Into<BluetoothStream> for SdpSession {
    fn into(self) -> BluetoothStream {
        unsafe {
            let fd = bluetooth_sys::sdp_get_socket(self.0);
            BluetoothStream::from_raw_fd(fd, BtProto::L2CAP)
        }
    }
}
