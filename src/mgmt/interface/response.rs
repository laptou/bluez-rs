use std::convert::TryInto;

use bytes::*;
use enumflags2::BitFlags;
use num_traits::FromPrimitive;

use crate::Address;
use crate::mgmt::client::{ConnectionInfo, ConnectionParams};
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
                0x0006 => ManagementEvent::NewSettings {
                    settings: BitFlags::from_bits_truncate(buf.get_u32_le()),
                },
                0x0007 => ManagementEvent::ClassOfDeviceChanged {
                    class: crate::interface::class::from_bytes(buf),
                },
                0x0008 => {
                    let name = buf.split_to(249).get_c_string();
                    let short_name = buf.get_c_string();

                    ManagementEvent::LocalNameChanged { name, short_name }
                }
                0x0009 => ManagementEvent::NewLinkKey {
                    store_hint: buf.get_bool(),
                    address: Address::from_slice(buf.split_to(6).as_ref()),
                    address_type: FromPrimitive::from_u8(buf.get_u8()).unwrap(),
                    key_type: FromPrimitive::from_u8(buf.get_u8()).unwrap(),
                    value: buf.split_to(16).as_ref().try_into().unwrap(),
                    pin_length: buf.get_u8(),
                },
                0x000A => ManagementEvent::NewLongTermKey {
                    store_hint: buf.get_bool(),
                    address: Address::from_slice(buf.split_to(6).as_ref()),
                    address_type: FromPrimitive::from_u8(buf.get_u8()).unwrap(),
                    key_type: FromPrimitive::from_u8(buf.get_u8()).unwrap(),
                    master: buf.get_u8(),
                    encryption_size: buf.get_u8(),
                    encryption_diversifier: buf.get_u16_le(),
                    random_number: buf.get_u64_le(),
                    value: buf.split_to(16).as_ref().try_into().unwrap(),
                },
                0x000B => ManagementEvent::DeviceConnected {
                    address: Address::from_slice(buf.split_to(6).as_ref()),
                    address_type: FromPrimitive::from_u8(buf.get_u8()).unwrap(),
                    flags: BitFlags::from_bits_truncate(buf.get_u32_le()),
                    eir_data: {
                        let len = buf.get_u16_le() as usize;
                        buf.split_to(len)
                    },
                },
                0x000C => ManagementEvent::DeviceDisconnected {
                    address: Address::from_slice(buf.split_to(6).as_ref()),
                    address_type: FromPrimitive::from_u8(buf.get_u8()).unwrap(),
                    reason: FromPrimitive::from_u8(buf.get_u8()).unwrap(),
                },
                0x000D => ManagementEvent::ConnectFailed {
                    address: Address::from_slice(buf.split_to(6).as_ref()),
                    address_type: FromPrimitive::from_u8(buf.get_u8()).unwrap(),
                    status: buf.get_u8(),
                },
                0x000E => ManagementEvent::PinCodeRequest {
                    address: Address::from_slice(buf.split_to(6).as_ref()),
                    address_type: FromPrimitive::from_u8(buf.get_u8()).unwrap(),
                    secure: buf.get_bool(),
                },
                0x000F => ManagementEvent::UserConfirmationRequest {
                    address: Address::from_slice(buf.split_to(6).as_ref()),
                    address_type: FromPrimitive::from_u8(buf.get_u8()).unwrap(),
                    confirm_hint: buf.get_bool(),
                    value: buf.get_u32_le(),
                },
                0x0010 => ManagementEvent::UserPasskeyRequest {
                    address: Address::from_slice(buf.split_to(6).as_ref()),
                    address_type: FromPrimitive::from_u8(buf.get_u8()).unwrap(),
                },
                0x0011 => ManagementEvent::AuthenticationFailed {
                    address: Address::from_slice(buf.split_to(6).as_ref()),
                    address_type: FromPrimitive::from_u8(buf.get_u8()).unwrap(),
                    status: buf.get_u8(),
                },
                0x0012 => ManagementEvent::DeviceFound {
                    address: Address::from_slice(buf.split_to(6).as_ref()),
                    address_type: FromPrimitive::from_u8(buf.get_u8()).unwrap(),
                    rssi: buf.get_i8(),
                    flags: BitFlags::from_bits_truncate(buf.get_u32_le()),
                    eir_data: {
                        let len = buf.get_u16_le() as usize;
                        buf.split_to(len)
                    },
                },
                0x0013 => ManagementEvent::Discovering {
                    address_type: BitFlags::from_bits_truncate(buf.get_u8()),
                    discovering: buf.get_bool(),
                },
                0x0014 => ManagementEvent::DeviceBlocked {
                    address: Address::from_slice(buf.split_to(6).as_ref()),
                    address_type: FromPrimitive::from_u8(buf.get_u8()).unwrap(),
                },
                0x0015 => ManagementEvent::DeviceUnblocked {
                    address: Address::from_slice(buf.split_to(6).as_ref()),
                    address_type: FromPrimitive::from_u8(buf.get_u8()).unwrap(),
                },
                0x0016 => ManagementEvent::DeviceUnpaired {
                    address: Address::from_slice(buf.split_to(6).as_ref()),
                    address_type: FromPrimitive::from_u8(buf.get_u8()).unwrap(),
                },
                0x0017 => ManagementEvent::PasskeyNotify {
                    address: Address::from_slice(buf.split_to(6).as_ref()),
                    address_type: FromPrimitive::from_u8(buf.get_u8()).unwrap(),
                    passkey: buf.get_u32_le(),
                    entered: buf.get_u8(),
                },
                0x0018 => ManagementEvent::NewIdentityResolvingKey {
                    store_hint: buf.get_bool(),
                    random_address: buf.get_address(),
                    address: buf.get_address(),
                    address_type: buf.get_primitive_u8(),
                    value: buf.get_u8x16(),
                },
                0x0019 => ManagementEvent::NewSignatureResolvingKey {
                    store_hint: buf.get_bool(),
                    address: buf.get_address(),
                    address_type: buf.get_primitive_u8(),
                    key_type: buf.get_primitive_u8(),
                    value: buf.get_u8x16(),
                },
                0x001A => ManagementEvent::DeviceAdded {
                    address: buf.get_address(),
                    address_type: buf.get_primitive_u8(),
                    action: buf.get_primitive_u8(),
                },
                0x001B => ManagementEvent::DeviceRemoved {
                    address: buf.get_address(),
                    address_type: buf.get_primitive_u8(),
                },
                0x001C => ManagementEvent::NewConnectionParams {
                    store_hint: buf.get_bool(),
                    param: ConnectionParams {
                        address: buf.get_address(),
                        address_type: buf.get_primitive_u8(),
                        min_connection_interval: buf.get_u16_le(),
                        max_connection_interval: buf.get_u16_le(),
                        connection_latency: buf.get_u16_le(),
                        supervision_timeout: buf.get_u16_le(),
                    },
                },
                0x001D => ManagementEvent::UnconfiguredIndexAdded,
                0x001E => ManagementEvent::UnconfiguredIndexRemoved,
                0x001F => ManagementEvent::NewConfigOptions {
                    missing_options: BitFlags::from_bits_truncate(buf.get_u32_le()),
                },
                0x0020 => ManagementEvent::ExtendedIndexAdded {
                    controller_type: buf.get_primitive_u8(),
                    controller_bus: buf.get_primitive_u8(),
                },
                0x0021 => ManagementEvent::ExtendedIndexRemoved {
                    controller_type: buf.get_primitive_u8(),
                    controller_bus: buf.get_primitive_u8(),
                },
                0x0022 => ManagementEvent::LocalOutOfBandExtDataUpdated {
                    address_type: buf.get_primitive_u8(),
                    eir_data: {
                        let len = buf.get_u16_le() as usize;
                        buf.split_to(len)
                    },
                },
                0x0023 => ManagementEvent::AdvertisingAdded {
                    instance: buf.get_u8(),
                },
                0x0024 => ManagementEvent::AdvertisingRemoved {
                    instance: buf.get_u8(),
                },
                0x0025 => ManagementEvent::ExtControllerInfoChanged {
                    eir_data: {
                        let len = buf.get_u16_le() as usize;
                        buf.split_to(len)
                    },
                },
                0x0026 => ManagementEvent::PhyConfigChanged {
                    selected_phys: BitFlags::from_bits_truncate(buf.get_u32_le()),
                },
                _ => todo!("throw error instead of panicking"),
            },
        })
    }
}
