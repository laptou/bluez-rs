use std::fmt::{Display, Formatter};

use num_traits::FromPrimitive;

use crate::Address;
use crate::bt;
use crate::mgmt::ManagementError;
use crate::util::*;

use super::request::ManagementRequest;
use super::super::socket::ManagementSocket;

bitflags! {
    pub struct ControllerSettings: u32 {
        const Powered = 1 << 1;
        const Connectable = 1 << 2;
        const FastConnectable = 1 << 3;
        const Discoverable = 1 << 4;
        const Pairable = 1 << 5;
        const LinkLevelSecurity = 1 << 6;
        const SecureSimplePairing = 1 << 7;
        const BREDR = 1 << 8;
        const HighSpeed = 1 << 9;
        const LE = 1 << 10;
        const Advertising = 1 << 11;
    }
}

impl Display for ControllerSettings {
    fn fmt(&self, f: &mut Formatter) -> Result<(), ::std::fmt::Error> {
        let mut tags = vec![];

        if self.contains(ControllerSettings::Powered) {
            tags.push("powered");
        }

        if self.contains(ControllerSettings::Connectable) {
            tags.push("connectable");
        }

        if self.contains(ControllerSettings::FastConnectable) {
            tags.push("fast-connectable");
        }

        if self.contains(ControllerSettings::Discoverable) {
            tags.push("discoverable");
        }

        if self.contains(ControllerSettings::Pairable) {
            tags.push("pairable");
        }

        if self.contains(ControllerSettings::LinkLevelSecurity) {
            tags.push("link-level-security");
        }

        if self.contains(ControllerSettings::SecureSimplePairing) {
            tags.push("secure-simple-pairing");
        }

        if self.contains(ControllerSettings::BREDR) {
            tags.push("br/edr");
        }

        if self.contains(ControllerSettings::HighSpeed) {
            tags.push("high-speed");
        }

        if self.contains(ControllerSettings::LE) {
            tags.push("low-energy");
        }

        write!(f, "{}", tags.join(" "))
    }
}

#[repr(u16)]
#[derive(FromPrimitive)]
pub enum ManagementCommand {
    ReadVersionInfo = 0x0001,
    ReadSupportedCommands,
    ReadControllerIndexList,
    ReadControllerInfo,
    SetPowered,
    SetDiscoverable,
    SetConnectable,
    SetFastConnectable,
    SetPairable,
    SetLinkSecurity,
    SetSecureSimplePairing,
    SetHighSpeed,
    SetLowEnergy,
    SetDeviceClass,
    SetLocalName,
    AddUUID,
    RemoveUUID,
    LoadLinkKeys,
    LoadLongTermKeys,
    Disconnect,
    GetConnections,
    PinCodeReply,
    PinCodeNegativeReply,
    SetIOCapability,
    PairDevice,
    CancelPairDevice,
    UnpairDevice,
    UserConfirmationReply,
    UserConfirmationNegativeReply,
    UserPasskeyReply,
    UserPasskeyNegativeReply,
    ReadLocalOutOfBand,
    AddRemoteOutOfBand,
    RemoveRemoteOutOfBand,
    StartDiscovery,
    StopDiscovery,
    ConfirmName,
    BlockDevice,
    UnblockDevice,
    SetDeviceID,
    SetAdvertising,
    SetBREDR,
    SetStaticAddress,
    SetScanParameters,
}

pub enum ManagementEvent {
    CommandComplete {
        opcode: ManagementCommand,
        status: ManagementCommandStatus,
        param: Box<Vec<u8>>,
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
        name: Option<String>,
        short_name: Option<String>,
    },
    NewLinkKey {
        store_hint: u8,
        address: Address,
        address_type: u8,
        key_type: u8,
        value: [u8; 16],
        pin_length: u8,
    },
    NewLongTermKey {
        store_hint: u8,
        address: Address,
        address_type: u8,
        authenticated: bool,
        master: u8,
        encryption_size: u8,
        encryption_diversifier: u16,
        random_number: [u8; 8],
        value: [u8; 16],
    },
    DeviceConnected {
        address: Address,
        address_type: u8,
        flags: u32,
        eir_data_length: u16,
        eir_data: Box<Vec<u8>>,
    },
    DeviceDisconnected {
        address: Address,
        address_type: u8,
        reason: u8,
    },
    ConnectFailed {
        address: Address,
        address_type: u8,
        status: u8,
    },
    PinCodeRequest {
        address: Address,
        address_type: u8,
        secure: bool,
    },
    UserConfirmationRequest {
        address: Address,
        address_type: u8,
        confirm_hint: bool,
        value: u32,
    },
    UserPasskeyRequest {
        address: Address,
        address_type: u8,
    },
    AuthenticationFailed {
        address: Address,
        address_type: u8,
        status: u8,
    },
    DeviceFound {
        address: Address,
        address_type: u8,
        rssi: u8,
        flags: u32,
        eir_data_length: u16,
        eir_data: Box<Vec<u8>>,
    },
    Discovering {
        address_type: u8,
        discovering: bool,
    },
    DeviceBlocked {
        address: Address,
        address_type: u8,
    },
    DeviceUnblocked {
        address: Address,
        address_type: u8,
    },
    DeviceUnpaired {
        address: Address,
        address_type: u8,
    },
    PasskeyNotify {
        address: Address,
        address_type: u8,
        passkey: u32,
        entered: u8,
    },
}

#[repr(u8)]
#[derive(FromPrimitive, ToPrimitive)]
pub enum ManagementCommandStatus {
    Success = 0x00,
    UnknownCommand = 0x01,
    NotConnected = 0x02,
    Failed = 0x03,
    ConnectFailed = 0x04,
    AuthenticationFailed = 0x05,
    NotPaired = 0x06,
    NoResources = 0x07,
    Timeout = 0x08,
    AlreadyConnected = 0x09,
    Busy = 0x0A,
    Rejected = 0x0B,
    NotSupported = 0x0C,
    InvalidParams = 0x0D,
    Disconnected = 0x0E,
    NotPowered = 0x0F,
    Cancelled = 0x10,
    InvalidIndex = 0x011,
}

#[repr(C)]
pub struct VersionResponse {
    pub version: u8,
    pub revision: u16,
}

pub fn get_version(
    sock: &ManagementSocket,
    timeout: i32,
) -> Result<VersionResponse, failure::Error> {
    let request = ManagementRequest {
        opcode: bt::MGMT_OP_READ_VERSION as u16,
        param: Box::new(vec![]),
        controller: 0xFFFF,
    };

    sock.send(request)?;
    let response = sock.receive(timeout)?;

    match response.event {
        ManagementEvent::CommandComplete {
            status,
            param,
            opcode,
        } => Ok(VersionResponse {
            version: param[0],
            revision: read_u16_le(&param, 1),
        }),
        _ => Err(ManagementError::Unknown.into()),
    }
}

pub fn get_controllers(sock: &ManagementSocket, timeout: i32) -> Result<Vec<u16>, failure::Error> {
    let request = ManagementRequest {
        opcode: bt::MGMT_OP_READ_INDEX_LIST as u16,
        param: Box::new(vec![]),
        controller: 0xFFFF,
    };

    sock.send(request)?;
    let response = sock.receive(timeout)?;

    match response.event {
        ManagementEvent::CommandComplete {
            status,
            param,
            opcode,
        } => {
            let num_controllers = read_u16_le(&param, 0) as usize;
            let mut vec = vec![];
            for i in 0..num_controllers {
                vec.push(read_u16_le(&param, 2 + i * 2));
            }
            Ok(vec)
        }
        _ => Err(ManagementError::Unknown.into()),
    }
}

pub struct ControllerInfo {
    pub address: Address,
    pub bluetooth_version: u8,
    pub manufacturer: [u8; 2],
    pub supported_settings: ControllerSettings,
    pub current_settings: ControllerSettings,
    pub class_of_device: [u8; 3],
    pub name: Option<String>,
    pub short_name: Option<String>,
}

pub fn get_controller_info(
    sock: &ManagementSocket,
    controller: u16,
    timeout: i32,
) -> Result<ControllerInfo, failure::Error> {
    let request = ManagementRequest {
        opcode: bt::MGMT_OP_READ_INFO as u16,
        param: Box::new(vec![]),
        controller,
    };

    sock.send(request)?;
    let response = sock.receive(timeout)?;

    match response.event {
        ManagementEvent::CommandComplete {
            status,
            param,
            opcode,
        } => {
            let address = Address::from_slice(&param[0..6]);
            let bluetooth_version = param[6];
            let mut manufacturer = [0u8; 2];
            manufacturer.copy_from_slice(&param[7..9]);
            let supported_settings =
                ControllerSettings::from_bits_truncate(read_u32_le(&param, 9));
            let current_settings = ControllerSettings::from_bits_truncate(read_u32_le(&param, 13));
            let mut class_of_device = [0u8; 3];
            class_of_device.copy_from_slice(&param[17..20]);
            let name = read_str(&param, 20, 249);
            let short_name = read_str(&param, 269, 11);
            Ok(ControllerInfo {
                address,
                bluetooth_version,
                manufacturer,
                supported_settings,
                current_settings,
                class_of_device,
                name,
                short_name,
            })
        }
        _ => Err(ManagementError::Unknown.into()),
    }
}

#[repr(u8)]
pub enum Discoverability {
    None = 0x0,
    General = 0x1,
    Limited = 0x2,
}

#[repr(C)]
pub struct DiscoverableResponse {
    current: u32,
}
