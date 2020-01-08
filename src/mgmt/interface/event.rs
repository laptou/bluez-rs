use std::ffi::OsString;

use bytes::Bytes;

use crate::Address;
use crate::mgmt::interface::{ManagementCommand, ManagementCommandStatus};
use crate::mgmt::interface::controller::ControllerSettings;

#[repr(u8)]
#[derive(Debug, Eq, PartialEq, FromPrimitive)]
pub enum AddressType {
    BrEdr = 0,
    LEPublic = 1,
    LERandom = 2,
}

/// Used to represent the version of the BlueZ management
/// interface that is in use.
pub struct ManagementVersion {
    pub version: u8,
    pub revision: u16,
}

#[derive(Debug)]
pub enum ManagementEvent {
    CommandComplete {
        opcode: ManagementCommand,
        status: ManagementCommandStatus,
        param: Bytes,
    },
    CommandStatus {
        opcode: ManagementCommand,
        status: ManagementCommandStatus,
    },
    ControllerError {
        code: u8,
    },
    IndexAdded,
    IndexRemoved,
    NewSettings {
        settings: ControllerSettings,
    },
    ClassOfDeviceChanged {
        class: [u8; 3],
    },
    LocalNameChanged {
        name: OsString,
        short_name: OsString,
    },
    NewLinkKey {
        store_hint: u8,
        address: Address,
        address_type: AddressType,
        key_type: u8,
        value: [u8; 16],
        pin_length: u8,
    },
    NewLongTermKey {
        store_hint: u8,
        address: Address,
        address_type: AddressType,
        authenticated: bool,
        master: u8,
        encryption_size: u8,
        encryption_diversifier: u16,
        random_number: [u8; 8],
        value: [u8; 16],
    },
    DeviceConnected {
        address: Address,
        address_type: AddressType,
        flags: u32,
        eir_data_length: u16,
        eir_data: Bytes,
    },
    DeviceDisconnected {
        address: Address,
        address_type: AddressType,
        reason: u8,
    },
    ConnectFailed {
        address: Address,
        address_type: AddressType,
        status: u8,
    },
    PinCodeRequest {
        address: Address,
        address_type: AddressType,
        secure: bool,
    },
    UserConfirmationRequest {
        address: Address,
        address_type: AddressType,
        confirm_hint: bool,
        value: u32,
    },
    UserPasskeyRequest {
        address: Address,
        address_type: AddressType,
    },
    AuthenticationFailed {
        address: Address,
        address_type: AddressType,
        status: u8,
    },
    DeviceFound {
        address: Address,
        address_type: AddressType,
        rssi: u8,
        flags: u32,
        eir_data_length: u16,
        eir_data: Bytes,
    },
    Discovering {
        address_type: AddressType,
        discovering: bool,
    },
    DeviceBlocked {
        address: Address,
        address_type: AddressType,
    },
    DeviceUnblocked {
        address: Address,
        address_type: AddressType,
    },
    DeviceUnpaired {
        address: Address,
        address_type: AddressType,
    },
    PasskeyNotify {
        address: Address,
        address_type: AddressType,
        passkey: u32,
        entered: u8,
    },
}
