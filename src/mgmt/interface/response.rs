use std::ffi::{CStr, CString};

use bytes::{Buf, Bytes, IntoBuf};
use num_traits::FromPrimitive;

use crate::Address;
use crate::mgmt::interface::{AddressType, ManagementCommand, ManagementCommandStatus};
use crate::mgmt::interface::event::ManagementEvent;
use crate::mgmt::ManagementError;
use crate::util::*;

pub struct ManagementResponse {
    pub event: ManagementEvent,
    pub controller: u16,
}

