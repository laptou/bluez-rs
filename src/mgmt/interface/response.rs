use std::convert::TryFrom;
use std::ffi::{CStr, CString};

use bytes::*;
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

impl ManagementResponse {
    pub fn parse<T: Buf>(mut buf: T) -> Result<Self, ManagementError> {
        let evt_code = buf.get_u16_le();
        let controller = buf.get_u16_le();
        buf.advance(2); // we already know param length

        Ok(ManagementResponse {
            controller,
            event: match evt_code {
                0x0001 | 0x0002 => {
                    let opcode = buf.get_u16_le();
                    let opcode = FromPrimitive::from_u16(opcode)
                        .ok_or(ManagementError::UnknownOpcode { opcode })?;

                    let status = buf.get_u8();
                    let status = FromPrimitive::from_u8(status)
                        .ok_or(ManagementError::UnknownStatus { status })?;

                    if evt_code == 0x0001 {
                        ManagementEvent::CommandComplete {
                            opcode,
                            status,
                            param: buf.to_bytes(),
                        }
                    } else {
                        ManagementEvent::CommandStatus {
                            opcode,
                            status,
                        }
                    }
                }
                0x0003 => ManagementEvent::ControllerError { code: buf.get_u8() },
                0x0004 => ManagementEvent::IndexAdded,
                0x0005 => ManagementEvent::IndexRemoved,
                0x0006 => ManagementEvent::NewSettings {
                    settings: unimplemented!(),
                },
                0x0007 => ManagementEvent::ClassOfDeviceChanged {
                    class: unimplemented!(),
                },
                0x0008 => {
                    let name = get_string(&mut buf, 249);
                    let short_name = get_string(&mut buf, 11);

                    ManagementEvent::LocalNameChanged { name, short_name }
                }
                _ => unimplemented!(),
            },
        })
    }
}
