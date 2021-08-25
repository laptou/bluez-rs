use std::{borrow::Borrow, ffi::CStr, fmt::Display, iter::FromIterator, marker::PhantomData};

use enumflags2::{bitflags, BitFlags};

use crate::{socket::BtProto, Address};

use super::stream::BluetoothStream;

#[repr(transparent)]
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
    item: PhantomData<&'a mut T>,
}

impl<'a, T> SdpList<'a, T> {
    pub fn create(item: &'a T) -> Self {
        SdpList {
            inner: unsafe {
                bluetooth_sys::sdp_list_append(std::ptr::null_mut(), item as *const T as *mut _)
            },
            item: PhantomData,
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

impl<'a, T> FromIterator<&'a T> for SdpList<'a, T> {
    fn from_iter<U: IntoIterator<Item = &'a T>>(iter: U) -> Self {
        let iter = iter.into_iter();
        let mut list = SdpList {
            inner: std::ptr::null_mut(),
            item: PhantomData,
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
pub struct SdpRecord(*mut bluetooth_sys::sdp_record_t);

impl SdpRecord {
    pub fn get(&self, attr_id: u16) -> &SdpData {
        let data = unsafe { bluetooth_sys::sdp_data_get(self.0, attr_id) };
        let data = data as *mut SdpData;
        unsafe { &*data }
    }

    pub fn get_access_protos(&self) -> std::io::Result<Vec<&SdpData>> {
        let mut list = std::ptr::null_mut();
        let res = unsafe { bluetooth_sys::sdp_get_access_protos(self.0, &mut list) };
        if res < 0 {
            return Err(std::io::Error::from_raw_os_error(res));
        }
        let list: &SdpList<SdpData> = unsafe { &*(list as *mut SdpList<SdpData>) };
        let list = list.iter().collect();
        Ok(list)
    }
}

impl Drop for SdpRecord {
    fn drop(&mut self) {
        unsafe { bluetooth_sys::sdp_record_free(self.0) }
    }
}

#[repr(transparent)]
pub struct SdpData(*mut bluetooth_sys::sdp_data_t);

impl SdpData {}

impl Drop for SdpData {
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
    ) -> std::io::Result<Vec<SdpRecord>> {
        let search_list: SdpList<SdpUuid> = search_list.into_iter().collect();
        let mut rsp_list: Option<SdpList<*mut bluetooth_sys::sdp_record_t>> = None;

        self.check_error(unsafe {
            bluetooth_sys::sdp_service_search_req(
                self.0,
                search_list.inner,
                max_rec_num,
                &mut rsp_list as *mut _ as *mut *mut _,
            )
        })?;

        let result = rsp_list
            .expect("response list is None")
            .iter_mut()
            .map(|item| SdpRecord(*item))
            .collect();

        Ok(result)
    }

    pub fn search_attr_req(
        &self,
        search_list: &[SdpUuid],
        req_type: SdpAttributeSpecification,
        attr_list: &[u32],
    ) -> std::io::Result<Vec<SdpRecord>> {
        let search_list: SdpList<SdpUuid> = search_list.into_iter().collect();
        let attr_list: SdpList<u32> = attr_list.into_iter().collect();
        let mut rsp_list: Option<SdpList<*mut bluetooth_sys::sdp_record_t>> = None;

        self.check_error(unsafe {
            bluetooth_sys::sdp_service_search_attr_req(
                self.0,
                search_list.inner,
                req_type as u32,
                attr_list.inner,
                &mut rsp_list as *mut _ as *mut *mut _,
            )
        })?;

        let result = rsp_list
            .expect("response list is None")
            .iter_mut()
            .map(|item| SdpRecord(*item))
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
