use bytes::*;
use enumflags2::BitFlags;
use num_traits::FromPrimitive;

use crate::management::client::ConnectionParams;
use crate::management::interface::controller::Controller;
use crate::management::interface::event::Event;
use crate::management::Error;
use crate::util::BufExt;
use crate::Address;

/// A response from the BlueZ management API. This can be a response to a
/// command that was issued, or an event that was sent in response to an outside
/// stimulus.
pub struct Response {
    pub event: Event,
    pub controller: Controller,
}

impl Response {
    pub fn parse<T: Buf>(mut buf: T) -> Result<Self, Error> {
        let evt_code = buf.get_u16_le();
        let controller = Controller(buf.get_u16_le());
        buf.advance(2); // we already know param length

        Ok(Response {
            controller,
            event: match evt_code {
                0x0001 | 0x0002 => {
                    let opcode = buf.get_u16_le();
                    let opcode =
                        FromPrimitive::from_u16(opcode).ok_or(Error::UnknownOpcode { opcode })?;

                    let status = buf.get_u8();
                    let status =
                        FromPrimitive::from_u8(status).ok_or(Error::UnknownStatus { status })?;

                    if evt_code == 0x0001 {
                        Event::CommandComplete {
                            opcode,
                            status,
                            param: buf.copy_to_bytes(buf.remaining()),
                        }
                    } else {
                        Event::CommandStatus { opcode, status }
                    }
                }
                0x0003 => Event::ControllerError { code: buf.get_u8() },
                0x0004 => Event::IndexAdded,
                0x0005 => Event::IndexRemoved,
                0x0006 => Event::NewSettings {
                    settings: BitFlags::from_bits_truncate(buf.get_u32_le()),
                },
                0x0007 => Event::ClassOfDeviceChanged {
                    class: super::device_class_from_buf(&mut buf),
                },
                0x0008 => {
                    let name = {
                        let mut arr = [0u8; 249];
                        buf.copy_to_slice(&mut arr[..]);
                        (&arr[..]).get_c_string()
                    };
                    let short_name = buf.get_c_string();

                    Event::LocalNameChanged { name, short_name }
                }
                0x0009 => Event::NewLinkKey {
                    store_hint: buf.get_bool(),
                    address: Address::from_buf(&mut buf),
                    address_type: FromPrimitive::from_u8(buf.get_u8()).ok_or(Error::InvalidData)?,
                    key_type: FromPrimitive::from_u8(buf.get_u8()).ok_or(Error::InvalidData)?,
                    value: buf.get_array_u8(),
                    pin_length: buf.get_u8(),
                },
                0x000A => Event::NewLongTermKey {
                    store_hint: buf.get_bool(),
                    address: Address::from_buf(&mut buf),
                    address_type: FromPrimitive::from_u8(buf.get_u8()).ok_or(Error::InvalidData)?,
                    key_type: FromPrimitive::from_u8(buf.get_u8()).ok_or(Error::InvalidData)?,
                    master: buf.get_u8(),
                    encryption_size: buf.get_u8(),
                    encryption_diversifier: buf.get_u16_le(),
                    random_number: buf.get_u64_le(),
                    value: buf.get_array_u8(),
                },
                0x000B => Event::DeviceConnected {
                    address: Address::from_buf(&mut buf),
                    address_type: FromPrimitive::from_u8(buf.get_u8()).ok_or(Error::InvalidData)?,
                    flags: BitFlags::from_bits_truncate(buf.get_u32_le()),
                    eir_data: {
                        let len = buf.get_u16_le() as usize;
                        buf.copy_to_bytes(len)
                    },
                },
                0x000C => Event::DeviceDisconnected {
                    address: Address::from_buf(&mut buf),
                    address_type: FromPrimitive::from_u8(buf.get_u8()).ok_or(Error::InvalidData)?,
                    reason: FromPrimitive::from_u8(buf.get_u8()).ok_or(Error::InvalidData)?,
                },
                0x000D => Event::ConnectFailed {
                    address: Address::from_buf(&mut buf),
                    address_type: FromPrimitive::from_u8(buf.get_u8()).ok_or(Error::InvalidData)?,
                    status: buf.get_u8(),
                },
                0x000E => Event::PinCodeRequest {
                    address: Address::from_buf(&mut buf),
                    address_type: FromPrimitive::from_u8(buf.get_u8()).ok_or(Error::InvalidData)?,
                    secure: buf.get_bool(),
                },
                0x000F => Event::UserConfirmationRequest {
                    address: Address::from_buf(&mut buf),
                    address_type: FromPrimitive::from_u8(buf.get_u8()).ok_or(Error::InvalidData)?,
                    confirm_hint: buf.get_bool(),
                    value: buf.get_u32_le(),
                },
                0x0010 => Event::UserPasskeyRequest {
                    address: Address::from_buf(&mut buf),
                    address_type: FromPrimitive::from_u8(buf.get_u8()).ok_or(Error::InvalidData)?,
                },
                0x0011 => Event::AuthenticationFailed {
                    address: Address::from_buf(&mut buf),
                    address_type: FromPrimitive::from_u8(buf.get_u8()).ok_or(Error::InvalidData)?,
                    status: buf.get_u8(),
                },
                0x0012 => Event::DeviceFound {
                    address: Address::from_buf(&mut buf),
                    address_type: FromPrimitive::from_u8(buf.get_u8()).ok_or(Error::InvalidData)?,
                    rssi: buf.get_i8(),
                    flags: BitFlags::from_bits_truncate(buf.get_u32_le()),
                    eir_data: {
                        let len = buf.get_u16_le() as usize;
                        buf.copy_to_bytes(len)
                    },
                },
                0x0013 => Event::Discovering {
                    address_type: BitFlags::from_bits_truncate(buf.get_u8()),
                    discovering: buf.get_bool(),
                },
                0x0014 => Event::DeviceBlocked {
                    address: Address::from_buf(&mut buf),
                    address_type: FromPrimitive::from_u8(buf.get_u8()).ok_or(Error::InvalidData)?,
                },
                0x0015 => Event::DeviceUnblocked {
                    address: Address::from_buf(&mut buf),
                    address_type: FromPrimitive::from_u8(buf.get_u8()).ok_or(Error::InvalidData)?,
                },
                0x0016 => Event::DeviceUnpaired {
                    address: Address::from_buf(&mut buf),
                    address_type: FromPrimitive::from_u8(buf.get_u8()).ok_or(Error::InvalidData)?,
                },
                0x0017 => Event::PasskeyNotify {
                    address: Address::from_buf(&mut buf),
                    address_type: FromPrimitive::from_u8(buf.get_u8()).ok_or(Error::InvalidData)?,
                    passkey: buf.get_u32_le(),
                    entered: buf.get_u8(),
                },
                0x0018 => Event::NewIdentityResolvingKey {
                    store_hint: buf.get_bool(),
                    random_address: buf.get_address(),
                    address: buf.get_address(),
                    address_type: buf.get_primitive_u8(),
                    value: buf.get_array_u8(),
                },
                0x0019 => Event::NewSignatureResolvingKey {
                    store_hint: buf.get_bool(),
                    address: buf.get_address(),
                    address_type: buf.get_primitive_u8(),
                    key_type: buf.get_primitive_u8(),
                    value: buf.get_array_u8(),
                },
                0x001A => Event::DeviceAdded {
                    address: buf.get_address(),
                    address_type: buf.get_primitive_u8(),
                    action: buf.get_primitive_u8(),
                },
                0x001B => Event::DeviceRemoved {
                    address: buf.get_address(),
                    address_type: buf.get_primitive_u8(),
                },
                0x001C => Event::NewConnectionParams {
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
                0x001D => Event::UnconfiguredIndexAdded,
                0x001E => Event::UnconfiguredIndexRemoved,
                0x001F => Event::NewConfigOptions {
                    missing_options: BitFlags::from_bits_truncate(buf.get_u32_le()),
                },
                0x0020 => Event::ExtendedIndexAdded {
                    controller_type: buf.get_primitive_u8(),
                    controller_bus: buf.get_primitive_u8(),
                },
                0x0021 => Event::ExtendedIndexRemoved {
                    controller_type: buf.get_primitive_u8(),
                    controller_bus: buf.get_primitive_u8(),
                },
                0x0022 => Event::LocalOutOfBandExtDataUpdated {
                    address_type: buf.get_primitive_u8(),
                    eir_data: {
                        let len = buf.get_u16_le() as usize;
                        buf.copy_to_bytes(len)
                    },
                },
                0x0023 => Event::AdvertisingAdded {
                    instance: buf.get_u8(),
                },
                0x0024 => Event::AdvertisingRemoved {
                    instance: buf.get_u8(),
                },
                0x0025 => Event::ExtControllerInfoChanged {
                    eir_data: {
                        let len = buf.get_u16_le() as usize;
                        buf.copy_to_bytes(len)
                    },
                },
                0x0026 => Event::PhyConfigChanged {
                    selected_phys: BitFlags::from_bits_truncate(buf.get_u32_le()),
                },
                0x0027 => Event::ExperimentalFeatureChanged {
                    uuid: buf.get_array_u8(),
                    flags: buf.get_u32_le(),
                },
                0x0028 => Event::DefaultSystemConfigChanged {
                    params: buf.get_tlv_map(),
                },
                0x0029 => Event::DefaultRuntimeConfigChanged {
                    params: buf.get_tlv_map(),
                },
                _ => return Err(Error::UnknownEventCode { evt_code }),
            },
        })
    }
}
