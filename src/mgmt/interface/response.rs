use std::convert::TryInto;

use bytes::*;
use num_traits::FromPrimitive;

use crate::Address;
use crate::mgmt::interface::controller::Controller;
use crate::mgmt::interface::event::ManagementEvent;
use crate::mgmt::ManagementError;
use crate::util::bytes_to_c_str;

pub struct ManagementResponse {
    pub event: ManagementEvent,
    pub controller: Controller,
}

impl ManagementResponse {
    pub fn parse<T: Buf>(mut buf: T) -> Result<Self, ManagementError> {
        let evt_code = buf.get_u16_le();
        let controller = Controller(buf.get_u16_le());
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
                        ManagementEvent::CommandStatus { opcode, status }
                    }
                }
                0x0003 => ManagementEvent::ControllerError { code: buf.get_u8() },
                0x0004 => ManagementEvent::IndexAdded,
                0x0005 => ManagementEvent::IndexRemoved,
                0x0006 => todo!("ManagementEvent::NewSettings"),
                0x0007 => todo!("ManagementEvent::ClassOfDeviceChanged"),
                0x0008 => {
                    let mut buf = buf.to_bytes();
                    let name = bytes_to_c_str(buf.split_to(249));
                    let short_name = bytes_to_c_str(buf);

                    ManagementEvent::LocalNameChanged { name, short_name }
                },
                0x0009 => {
                    let mut buf = buf.to_bytes();
                    ManagementEvent::NewLinkKey {
                        store_hint: buf.get_u8() as bool,
                        address: Address::from_slice(buf.split_to(6).as_ref()),
                        address_type: FromPrimitive::from_u8(buf.get_u8()).unwrap(),
                        key_type: FromPrimitive::from_u8(buf.get_u8()).unwrap(),
                        value: buf.split_to(16).as_ref().try_into().unwrap(),
                        pin_length: buf.get_u8()
                    }
                }
                _ => todo!("throw error instead of panicking"),
            },
        })
    }
}
