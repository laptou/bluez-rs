use std::convert::TryInto;

use bytes::*;
use num_traits::FromPrimitive;

use crate::Address;
use crate::mgmt::interface::controller::Controller;
use crate::mgmt::interface::event::ManagementEvent;
use crate::mgmt::ManagementError;
use crate::util::BufExt2;

pub struct ManagementResponse {
    pub event: ManagementEvent,
    pub controller: Controller,
}

impl ManagementResponse {
    pub fn parse<T: Buf>(mut buf: T) -> Result<Self, ManagementError> {
        let evt_code = buf.get_u16_le();
        let controller = Controller(buf.get_u16_le());
        buf.advance(2); // we already know param length
        let mut buf = buf.to_bytes();

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
                    let name = buf.split_to(249).get_c_string();
                    let short_name = buf.get_c_string();

                    ManagementEvent::LocalNameChanged { name, short_name }
                },
                0x0009 => {
                    ManagementEvent::NewLinkKey {
                        store_hint: buf.get_bool(),
                        address: Address::from_slice(buf.split_to(6).as_ref()),
                        address_type: FromPrimitive::from_u8(buf.get_u8()).unwrap(),
                        key_type: FromPrimitive::from_u8(buf.get_u8()).unwrap(),
                        value: buf.split_to(16).as_ref().try_into().unwrap(),
                        pin_length: buf.get_u8()
                    }
                },
                0x000A => {
                    ManagementEvent::NewLongTermKey {
                        store_hint: buf.get_bool(),
                        address: Address::from_slice(buf.split_to(6).as_ref()),
                        address_type: FromPrimitive::from_u8(buf.get_u8()).unwrap(),
                        key_type: FromPrimitive::from_u8(buf.get_u8()).unwrap(),
                        master: buf.get_u8(),
                        encryption_size: buf.get_u8(),
                        encryption_diversifier: buf.get_u16_le(),
                        random_number: buf.get_u64_le(),
                        value: buf.split_to(16).as_ref().try_into().unwrap(),
                    }
                },
                0x000B => {
                    ManagementEvent::DeviceConnected {
                        address: Address::from_slice(buf.split_to(6).as_ref()),
                        address_type: FromPrimitive::from_u8(buf.get_u8()).unwrap(),
                        flags: FromPrimitive::from_u32(buf.get_u32_le()).unwrap(),
                        eir_data: {
                            let len = buf.get_u16_le() as usize;
                            buf.split_to(len)
                        }
                    }
                },
                0x000C => {
                    ManagementEvent::DeviceDisconnected {
                        address: Address::from_slice(buf.split_to(6).as_ref()),
                        address_type: FromPrimitive::from_u8(buf.get_u8()).unwrap(),
                        reason: FromPrimitive::from_u8(buf.get_u8()).unwrap(),
                    }
                },
                0x000D => {
                    ManagementEvent::ConnectFailed {
                        address: Address::from_slice(buf.split_to(6).as_ref()),
                        address_type: FromPrimitive::from_u8(buf.get_u8()).unwrap(),
                        status: buf.get_u8()
                    }
                },
                0x000E => {
                    ManagementEvent::PinCodeRequest {
                        address: Address::from_slice(buf.split_to(6).as_ref()),
                        address_type: FromPrimitive::from_u8(buf.get_u8()).unwrap(),
                        secure: buf.get_bool()
                    }
                },
                0x000F => {
                    ManagementEvent::UserConfirmationRequest {
                        address: Address::from_slice(buf.split_to(6).as_ref()),
                        address_type: FromPrimitive::from_u8(buf.get_u8()).unwrap(),
                        confirm_hint: buf.get_bool(),
                        value: buf.get_u32_le()
                    }
                },
                0x0010 => {
                    ManagementEvent::UserPasskeyRequest {
                        address: Address::from_slice(buf.split_to(6).as_ref()),
                        address_type: FromPrimitive::from_u8(buf.get_u8()).unwrap(),
                    }
                },
                0x0011 => {
                    ManagementEvent::AuthenticationFailed {
                        address: Address::from_slice(buf.split_to(6).as_ref()),
                        address_type: FromPrimitive::from_u8(buf.get_u8()).unwrap(),
                        status: buf.get_u8()
                    }
                },
                0x0012 => {
                    ManagementEvent::DeviceFound {
                        address: Address::from_slice(buf.split_to(6).as_ref()),
                        address_type: FromPrimitive::from_u8(buf.get_u8()).unwrap(),
                        rssi: buf.get_i8(),
                        flags: FromPrimitive::from_u32(buf.get_u32_le()).unwrap(),
                        eir_data: {
                            let len = buf.get_u16_le() as usize;
                            buf.split_to(len)
                        }
                    }
                },
                0x0013 => {
                    ManagementEvent::Discovering {
                        address_type: FromPrimitive::from_u8(buf.get_u8()).unwrap(),
                        discovering: buf.get_bool()
                    }
                },
                0x0014 => {
                    ManagementEvent::DeviceBlocked {
                        address: Address::from_slice(buf.split_to(6).as_ref()),
                        address_type: FromPrimitive::from_u8(buf.get_u8()).unwrap(),
                    }
                },
                0x0015 => {
                    ManagementEvent::DeviceUnblocked {
                        address: Address::from_slice(buf.split_to(6).as_ref()),
                        address_type: FromPrimitive::from_u8(buf.get_u8()).unwrap(),
                    }
                },
                0x0016 => {
                    ManagementEvent::DeviceUnpaired {
                        address: Address::from_slice(buf.split_to(6).as_ref()),
                        address_type: FromPrimitive::from_u8(buf.get_u8()).unwrap(),
                    }
                },
                0x0017 => {
                    ManagementEvent::PasskeyNotify {
                        address: Address::from_slice(buf.split_to(6).as_ref()),
                        address_type: FromPrimitive::from_u8(buf.get_u8()).unwrap(),
                        passkey: buf.get_u32_le(),
                        entered: buf.get_u8()
                    }
                },
                0x0018 => {
                    ManagementEvent::NewIdentityResolvingKey {
                        store_hint: buf.get_bool(),
                        random_address: buf.get_address(),
                        address: buf.get_address(),
                        address_type: buf.get_primitive_u8(),
                        value: buf.get_u8x16(),
                    }
                },
                _ => todo!("throw error instead of panicking"),
            },
        })
    }
}
