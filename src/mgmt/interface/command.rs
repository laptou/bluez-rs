use std::fmt::{Display, Formatter};

use num_traits::FromPrimitive;

use crate::Address;
use crate::mgmt::ManagementError;
use crate::util::*;

use super::class::{DeviceClass, ServiceClass};
use super::request::ManagementRequest;
use super::response::ManagementResponse;
use super::super::socket::ManagementSocket;

bitflags! {
    pub struct ControllerSettings: u32 {
        const Powered = 1 << 0;
        const Connectable = 1 << 1;
        const FastConnectable = 1 << 2;
        const Discoverable = 1 << 3;
        const Pairable = 1 << 4;
        const LinkLevelSecurity = 1 << 5;
        const SecureSimplePairing = 1 << 6;
        const BREDR = 1 << 7;
        const HighSpeed = 1 << 8;
        const LE = 1 << 9;
        const Advertising = 1 << 10;
        const SecureConnection = 1 << 11;
        const DebugKeys = 1 << 12;
        const Privacy = 1 << 13;
        const Configuration = 1 << 14;
        const StaticAddress = 1 << 15;
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

        if self.contains(ControllerSettings::Advertising) {
            tags.push("advertising");
        }

        if self.contains(ControllerSettings::SecureConnection) {
            tags.push("secure-connection");
        }

        if self.contains(ControllerSettings::DebugKeys) {
            tags.push("debug-keys");
        }

        if self.contains(ControllerSettings::Privacy) {
            tags.push("privacy");
        }

        if self.contains(ControllerSettings::StaticAddress) {
            tags.push("static-address");
        }

        write!(f, "{}", tags.join(" "))
    }
}

#[repr(u16)]
#[derive(FromPrimitive, ToPrimitive, Copy, Clone, Debug)]
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

#[repr(u8)]
#[derive(Debug, Eq, PartialEq, FromPrimitive)]
pub enum AddressType {
    BrEdr = 0,
    LEPublic = 1,
    LERandom = 2,
}

#[derive(Debug)]
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
        eir_data: Box<Vec<u8>>,
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
        eir_data: Box<Vec<u8>>,
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

#[repr(u8)]
#[derive(FromPrimitive, ToPrimitive, Copy, Clone, Debug)]
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
    InvalidIndex = 0x11,
    RFKilled = 0x12,
    AlreadyPaired = 0x13,
    PermissionDenied = 0x14,
}

impl ::std::fmt::LowerHex for ManagementCommandStatus {
    fn fmt(&self, f: &mut Formatter) -> Result<(), ::std::fmt::Error> {
        write!(f, "{:x}", *self as u8)
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Controller(u16);

impl Display for Controller {
    fn fmt(&self, f: &mut Formatter) -> Result<(), ::std::fmt::Error> {
        write!(f, "hci{}", self.0)
    }
}

trait ResponseExt {
    fn map_success<
        T,
        R: ::std::convert::From<ManagementError>,
        F: Fn(Box<Vec<u8>>, ManagementCommandStatus, ManagementCommand) -> Result<T, R>,
    >(
        self,
        complete: F,
    ) -> Result<T, R>;
}

impl ResponseExt for ManagementResponse {
    fn map_success<
        T,
        R: ::std::convert::From<ManagementError>,
        F: Fn(Box<Vec<u8>>, ManagementCommandStatus, ManagementCommand) -> Result<T, R>,
    >(
        self,
        complete: F,
    ) -> Result<T, R> {
        match self.event {
            ManagementEvent::CommandComplete {
                param,
                status,
                opcode,
            } => match status {
                ManagementCommandStatus::Success => complete(param, status, opcode),
                _ => Err(ManagementError::CommandError { status, opcode }.into()),
            },
            ManagementEvent::CommandStatus { status, opcode } => {
                Err(ManagementError::CommandError { status, opcode }.into())
            }
            _ => Err(ManagementError::Unknown.into()),
        }
    }
}

#[inline(always)]
fn await_request(
    socket: &ManagementSocket,
    opcode: ManagementCommand,
    param: Vec<u8>,
    controller: Controller,
    timeout: i32,
) -> Result<ManagementResponse, ::failure::Error> {
    socket.send(ManagementRequest {
        opcode: opcode,
        param: Box::new(param),
        controller: controller.0,
    })?;
    socket.receive(timeout)
}

fn map_controller_settings(
    param: Box<Vec<u8>>,
    _status: ManagementCommandStatus,
    _command: ManagementCommand,
) -> Result<ControllerSettings, failure::Error> {
    Ok(ControllerSettings::from_bits_truncate(read_u32_le(
        &param, 0,
    )))
}

/// Used to represent the version of the BlueZ management
/// interface that is in use.
pub struct Version {
    pub version: u8,
    pub revision: u16,
}

/// Gets the version of the BlueZ management interface
/// that is in use.
pub fn get_version(sock: &ManagementSocket, timeout: i32) -> Result<Version, failure::Error> {
    let param = vec![];

    await_request(
        sock,
        ManagementCommand::ReadVersionInfo,
        param,
        Controller(0xFFFF),
        timeout,
    )?
        .map_success(|param, _, _| {
            Ok(Version {
                version: param[0],
                revision: read_u16_le(&param, 1),
            })
    })
}

/// Gets all of the controllers available on the current
/// device.
pub fn get_controllers(
    sock: &ManagementSocket,
    timeout: i32,
) -> Result<Vec<Controller>, failure::Error> {
    await_request(
        sock,
        ManagementCommand::ReadControllerIndexList,
        vec![],
        Controller(0xFFFF),
        timeout,
    )?
        .map_success(|param, _, _| {
            let num_controllers = read_u16_le(&param, 0) as usize;
            let mut vec = vec![];
            for i in 0..num_controllers {
                vec.push(Controller(read_u16_le(&param, 2 + i * 2)));
            }
            Ok(vec)
        })
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

/// Gets information about a specified controller.
pub fn get_controller_info(
    sock: &ManagementSocket,
    controller: Controller,
    timeout: i32,
) -> Result<ControllerInfo, failure::Error> {
    await_request(
        sock,
        ManagementCommand::ReadControllerInfo,
        vec![],
        controller,
        timeout,
    )?
        .map_success(|param, _, _| {
            let address = Address::from_slice(&param[0..6]);
            let bluetooth_version = param[6];
            let mut manufacturer = [0u8; 2];
            manufacturer.copy_from_slice(&param[7..9]);
            let supported_settings = ControllerSettings::from_bits_truncate(read_u32_le(&param, 9));
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
        })
}

/// Power on or off a controller.
/// If the discoverable setting is activated with a timeout, then
///	switching the controller off will disable discoverability
/// and discard the timeout.
pub fn set_powered(
    sock: &ManagementSocket,
    controller: Controller,
    powered: bool,
    timeout: i32,
) -> Result<ControllerSettings, failure::Error> {
    await_request(
        sock,
        ManagementCommand::SetPowered,
        vec![powered as u8],
        controller,
        timeout,
    )?
        .map_success(map_controller_settings)
}

#[repr(u8)]
pub enum Discoverability {
    None = 0x0,
    General = 0x1,
    Limited = 0x2,
}

/// Timeout is specified in seconds.
/// Set timeout to 0 to disable it.
/// For limited discoverability, the timeout is required.
/// Enabling discoverability while connectability is disabled
/// will error with Rejected.
/// This setting can be used when the controller is not powered,
/// unless a timeout is used, in which case it  will error with
/// Not Powered.
pub fn set_discoverable(
    sock: &ManagementSocket,
    controller: Controller,
    discoverable: Discoverability,
    discoverable_timeout: u16,
    timeout: i32,
) -> Result<ControllerSettings, failure::Error> {
    let mut param = vec![discoverable as u8];
    param.extend_from_slice(&u16::to_le_bytes(discoverable_timeout));

    await_request(
        sock,
        ManagementCommand::SetDiscoverable,
        param,
        controller,
        timeout,
    )?
        .map_success(map_controller_settings)
}

/// This command is available for BR/EDR, LE-only and also dual
///	mode controllers. For BR/EDR is changes the page scan setting
///	and for LE controllers it changes the advertising type. For
///	dual mode controllers it affects both settings.
///
///	For LE capable controllers the connectable setting takes effect
///	when advertising is enabled (peripheral) or when directed
///	advertising events are received (central).
///
///	This command can be used when the controller is not powered and
///	all settings will be programmed once powered.
///
///	When switching connectable off, it will also switch off the
///	discoverable setting. Switching connectable back on will not
///	restore a previous discoverable. It will stay off and needs
///	to be manually switched back on.
///
///	When switching connectable off, it will expire a discoverable
///	setting with a timeout.
///
///	This setting does not affect known devices from Add Device
///	command. These devices are always allowed to connect.
pub fn set_connectable(
    sock: &ManagementSocket,
    controller: Controller,
    connectable: bool,
    timeout: i32,
) -> Result<ControllerSettings, failure::Error> {
    await_request(
        sock,
        ManagementCommand::SetConnectable,
        vec![connectable as u8],
        controller,
        timeout,
    )?
        .map_success(map_controller_settings)
}

/// This command is used to set the controller into a connectable
///	state where the page scan parameters have been set in a way to
///	favor faster connect times with the expense of higher power
///	consumption.
///
/// This command is only available for BR/EDR capable controllers
///	(e.g. not for single-mode LE ones). It will return Not Supported
///	otherwise.
///
///	This command can be used when the controller is not powered and
///	all settings will be programmed once powered.
///
///	The setting will be remembered during power down/up toggles.
pub fn set_fast_connectable(
    sock: &ManagementSocket,
    controller: Controller,
    fast_connectable: bool,
    timeout: i32,
) -> Result<ControllerSettings, failure::Error> {
    await_request(
        sock,
        ManagementCommand::SetFastConnectable,
        vec![fast_connectable as u8],
        controller,
        timeout,
    )?
        .map_success(map_controller_settings)
}

/// This command can be used when the controller is not powered and
///	all settings will be programmed once powered.
///
///	Turning pairable on will not automatically switch the controller
///	into connectable mode. That needs to be done separately.
///
///	The setting will be remembered during power down/up toggles.
pub fn set_pairable(
    sock: &ManagementSocket,
    controller: Controller,
    pairable: bool,
    timeout: i32,
) -> Result<ControllerSettings, failure::Error> {
    await_request(
        sock,
        ManagementCommand::SetPairable,
        vec![pairable as u8],
        controller,
        timeout,
    )?
        .map_success(map_controller_settings)
}

///	Enable or disable link-level security, also known as Security Mode 3.
/// When this is enabled, the connection is encrypted and pairing is required
/// in order to communicate with a device.
///
/// This command is only available for BR/EDR capable controllers
///	(e.g. not for single-mode LE ones). It will return Not Supported
///	otherwise.
///
///	This command can be used when the controller is not powered and
///	all settings will be programmed once powered.
pub fn set_sm3(
    sock: &ManagementSocket,
    controller: Controller,
    enabled: bool,
    timeout: i32,
) -> Result<ControllerSettings, failure::Error> {
    await_request(
        sock,
        ManagementCommand::SetLinkSecurity,
        vec![enabled as u8],
        controller,
        timeout,
    )?
        .map_success(map_controller_settings)
}

/// This command is only available for BR/EDR capable controllers
///	supporting the core specification version 2.1 or greater
///	(e.g. not for single-mode LE controllers or pre-2.1 ones).
///
///	This command can be used when the controller is not powered and
///	all settings will be programmed once powered.
pub fn set_ssp(
    sock: &ManagementSocket,
    controller: Controller,
    enabled: bool,
    timeout: i32,
) -> Result<ControllerSettings, failure::Error> {
    await_request(
        sock,
        ManagementCommand::SetSecureSimplePairing,
        vec![enabled as u8],
        controller,
        timeout,
    )?
        .map_success(map_controller_settings)
}

/// This command is only available for BR/EDR capable controllers
///	(e.g. not for single-mode LE ones).
///
///	This command can be used when the controller is not powered and
///	all settings will be programmed once powered.
///
///	To enable High Speed support, it is required that Secure Simple
///	Pairing support is enabled first. High Speed support is not
///	possible for connections without Secure Simple Pairing.
///
///	When switching Secure Simple Pairing off, the support for High
///	Speed will be switched off as well. Switching Secure Simple
///	Pairing back on, will not re-enable High Speed support. That
///	needs to be done manually.
pub fn set_high_speed(
    sock: &ManagementSocket,
    controller: Controller,
    enabled: bool,
    timeout: i32,
) -> Result<ControllerSettings, failure::Error> {
    await_request(
        sock,
        ManagementCommand::SetHighSpeed,
        vec![enabled as u8],
        controller,
        timeout,
    )?
        .map_success(map_controller_settings)
}

/// This command is only available for LE capable controllers and
///	will yield in a Not Supported error otherwise.
///
///	This command can be used when the controller is not powered and
///	all settings will be programmed once powered.
///
///	In case the kernel subsystem does not support Low Energy or the
///	controller does not either, the command will fail regardless.
///
///	Disabling LE support will permanently disable and remove all
///	advertising instances configured with the Add Advertising
///	command. Advertising Removed events will be issued accordingly.
pub fn set_low_energy(
    sock: &ManagementSocket,
    controller: Controller,
    enabled: bool,
    timeout: i32,
) -> Result<ControllerSettings, failure::Error> {
    await_request(
        sock,
        ManagementCommand::SetLowEnergy,
        vec![enabled as u8],
        controller,
        timeout,
    )?
        .map_success(map_controller_settings)
}

/// This command is used to set the major and minor device class for
///	BR/EDR capable controllers.
///
///	This command will also implicitly disable caching of pending CoD
///	and EIR updates.
///
///	This command is only available for BR/EDR capable controllers
///	(e.g. not for single-mode LE ones).
///
///	This command can be used when the controller is not powered and
///	all settings will be programmed once powered.
///
///	In case the controller is powered off, Unknown will be returned
///	for the class of device parameter. And after power on the new
///	value will be announced via class of device changed event.
pub fn set_device_class(
    sock: &ManagementSocket,
    controller: Controller,
    class: DeviceClass,
    timeout: i32,
) -> Result<(DeviceClass, Vec<ServiceClass>), failure::Error> {
    await_request(
        sock,
        ManagementCommand::SetDeviceClass,
        super::class::to_u16(class).to_le_bytes().to_vec(),
        controller,
        timeout,
    )?
        .map_success(|param, _, _| {
            let mut cod = [0u8; 3];
            cod.copy_from_slice(&param);
            Ok(super::class::from_bytes(cod))
        })
}

#[derive(Debug)]
pub struct Name {
    pub name: String,
    pub short_name: Option<String>,
}

/// This command is used to set the local name of a controller. The
///	command parameters also include a short name which will be used
///	in case the full name doesn't fit within EIR/AD data.
///
///	The name parameters need to always end with a null byte (failure
///	to do so will cause the command to fail).
///
///	This command can be used when the controller is not powered and
///	all settings will be programmed once powered.
///
///	The values of name and short name will be remembered when
///	switching the controller off and back on again. So the name
///	and short name only have to be set once when a new controller
///	is found and will stay until removed.
pub fn set_name(
    sock: &ManagementSocket,
    controller: Controller,
    name: &str,
    short_name: Option<&str>,
    timeout: i32,
) -> Result<Name, failure::Error> {
    let name_bytes = name.as_bytes();
    let short_name_bytes = match short_name {
        Some(s) => s.as_bytes(),
        None => &[],
    };

    if name_bytes.len() > 248 {
        return Err(ManagementError::NameTooLong {
            name: unsafe { String::from_utf8_unchecked(name_bytes.to_vec()) },
            maxlen: 248,
        }
        .into());
    }

    if short_name_bytes.len() > 10 {
        return Err(ManagementError::NameTooLong {
            name: unsafe { String::from_utf8_unchecked(short_name_bytes.to_vec()) }, // unwrap is safe b/c obviously short name is not None
            maxlen: 10,
        }
        .into());
    }

    let mut param = vec![0; 260];
    param.splice(0..name_bytes.len(), name_bytes.iter().cloned());
    param.splice(
        249..249 + short_name_bytes.len(),
        short_name_bytes.iter().cloned(),
    );

    await_request(
        sock,
        ManagementCommand::SetLocalName,
        param,
        controller,
        timeout,
    )?
        .map_success(|param, _, _| {
            let name = read_str(&param, 0, 249).unwrap_or("".to_owned());
            let short_name = read_str(&param, 249, 11);
            Ok(Name { name, short_name })
        })
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Uuid([u8; 16]);

impl Display for Uuid {
    fn fmt(&self, f: &mut Formatter) -> Result<(), ::std::fmt::Error> {
        write!(
            f,
            "{:x}{:x}{:x}{:x}-{:x}{:x}-{:x}{:x}-{:x}{:x}-{:x}{:x}{:x}{:x}{:x}{:x}",
            self.0[0],
            self.0[1],
            self.0[2],
            self.0[3],
            self.0[4],
            self.0[5],
            self.0[6],
            self.0[7],
            self.0[8],
            self.0[9],
            self.0[10],
            self.0[11],
            self.0[12],
            self.0[13],
            self.0[14],
            self.0[15],
        )
    }
}

/// This command is used to add a UUID to be published in EIR data.
///	The accompanied SVC_Hint parameter is used to tell the kernel
///	whether the service class bits of the Class of Device value need
///	modifying due to this UUID.
///
///	This command can be used when the controller is not powered and
///	all settings will be programmed once powered.
///
///	In case the controller is powered off, 0x000000 will be returned
///	for the class of device parameter. And after power on the new
///	value will be announced via class of device changed event.
pub fn add_uuid(
    sock: &ManagementSocket,
    controller: Controller,
    uuid: Uuid,
    service_hint: u8,
    timeout: i32,
) -> Result<(DeviceClass, Vec<ServiceClass>), failure::Error> {
    let mut param = vec![];
    param.extend_from_slice(&uuid.0);
    param.push(service_hint);

    await_request(sock, ManagementCommand::AddUUID, param, controller, timeout)?.map_success(
        |param, _, _| {
            let mut cod = [0u8; 3];
            cod.copy_from_slice(&param);
            Ok(super::class::from_bytes(cod))
        },
    )
}

/// This command is used to remove a UUID previously added using the
///	Add UUID command.
///
///	When the UUID parameter is an empty UUID (16 x 0x00), then all
///	previously loaded UUIDs will be removed.
///
///	This command can be used when the controller is not powered and
///	all settings will be programmed once powered.
///
///	In case the controller is powered off, 0x000000 will be returned
///	for the class of device parameter. And after power on the new
///	value will be announced via class of device changed event.
pub fn remove_uuid(
    sock: &ManagementSocket,
    controller: Controller,
    uuid: Uuid,
    timeout: i32,
) -> Result<(DeviceClass, Vec<ServiceClass>), failure::Error> {
    await_request(
        sock,
        ManagementCommand::RemoveUUID,
        uuid.0.to_vec(),
        controller,
        timeout,
    )?
        .map_success(|param, _, _| {
            let mut cod = [0u8; 3];
            cod.copy_from_slice(&param);
            Ok(super::class::from_bytes(cod))
        })
}

#[repr(u8)]
pub enum LinkKeyType {
    Combination = 0x00,
    LocalUnit,
    RemoteUnit,
    DebugCombination,
    UnauthenticatedCombinationP192,
    AuthenticatedCombinationP192,
    ChangedCombination,
    UnauthenticatedCombinationP256,
    AuthenticatedCombinationP256,
}

pub struct LinkKey {
    address: Address,
    address_type: AddressType,
    key_type: LinkKeyType,
    value: [u8; 16],
    pin_length: u8,
}

pub fn load_link_keys(
    sock: &ManagementSocket,
    controller: Controller,
    keys: Vec<LinkKey>,
    debug: bool,
    timeout: i32,
) -> Result<(), failure::Error> {
    let mut param = vec![];
    param.push(debug as u8);
    param.extend_from_slice(&(keys.len() as u16).to_le_bytes());

    for key in keys {
        param.extend_from_slice(&key.address.bytes);
        param.push(key.address_type as u8);
        param.push(key.key_type as u8);
        param.extend_from_slice(&key.value);
        param.push(key.pin_length);
    }

    await_request(
        sock,
        ManagementCommand::LoadLinkKeys,
        param,
        controller,
        timeout,
    )?
        .map_success(|_, _, _| Ok(()))
}

#[repr(u8)]
pub enum LongTermKeyType {
    Unauthenticated = 0,
    Authenticated,
}

pub struct LongTermKey {
    address: Address,
    address_type: AddressType,
    key_type: LongTermKeyType,
    master: u8,
    encryption_size: u8,
    encryption_diversifier: [u8; 2],
    random_number: u64,
    value: [u8; 16],
}

pub fn load_long_term_keys(
    sock: &ManagementSocket,
    controller: Controller,
    keys: Vec<LongTermKey>,
    timeout: i32,
) -> Result<(), failure::Error> {
    let mut param = vec![];
    param.extend_from_slice(&(keys.len() as u16).to_le_bytes());

    for key in keys {
        param.extend_from_slice(&key.address.bytes);
        param.push(key.address_type as u8);
        param.push(key.key_type as u8);
        param.push(key.master);
        param.push(key.encryption_size);
        param.extend_from_slice(&key.encryption_diversifier);
        param.extend_from_slice(&key.random_number.to_le_bytes());
        param.extend_from_slice(&key.value);
    }

    await_request(
        sock,
        ManagementCommand::LoadLinkKeys,
        param,
        controller,
        timeout,
    )?
        .map_success(|_, _, _| Ok(()))
}

pub fn disconnect(
    sock: &ManagementSocket,
    controller: Controller,
    address: Address,
    address_type: AddressType,
    timeout: i32,
) -> Result<(Address, AddressType), failure::Error> {
    let mut param = vec![];
    param.extend_from_slice(&address.bytes);
    param.push(address_type as u8);

    await_request(
        sock,
        ManagementCommand::Disconnect,
        param,
        controller,
        timeout,
    )?
        .map_success(|param, _, _| {
            let addr = Address::from_slice(&param[0..6]);
            let addr_type: AddressType = FromPrimitive::from_u8(param[6])
                .ok_or::<failure::Error>(ManagementError::Unknown.into())?;
            Ok((addr, addr_type))
        })
}

pub fn get_connections(
    sock: &ManagementSocket,
    controller: Controller,
    timeout: i32,
) -> Result<Vec<(Address, AddressType)>, failure::Error> {
    await_request(
        sock,
        ManagementCommand::GetConnections,
        vec![],
        controller,
        timeout,
    )?
        .map_success(|param, _, _| {
            let count = read_u16_le(&param, 0) as usize;
            let mut addrs = vec![];

            for i in 0..count {
                let offset = 2 + i * 7;
                let addr = Address::from_slice(&param[offset..offset + 6]);
                let addr_type: AddressType = FromPrimitive::from_u8(param[offset + 6])
                    .ok_or::<failure::Error>(ManagementError::Unknown.into())?;
                addrs.push((addr, addr_type));
            }

            Ok(addrs)
        })
}

pub fn pin_code_reply(
    sock: &ManagementSocket,
    controller: Controller,
    address: Address,
    address_type: AddressType,
    pin: Vec<u8>,
    timeout: i32,
) -> Result<(Address, AddressType), failure::Error> {
    if pin.len() > 16 {
        return Err(ManagementError::PinCodeTooLong { maxlen: 16 }.into());
    }

    let mut param = vec![];
    param.extend_from_slice(&address.bytes);
    param.push(address_type as u8);
    param.push(pin.len() as u8);
    param.extend(pin);

    await_request(
        sock,
        ManagementCommand::PinCodeReply,
        param,
        controller,
        timeout,
    )?
        .map_success(|param, _, _| {
            let addr = Address::from_slice(&param[0..6]);
            let addr_type: AddressType = FromPrimitive::from_u8(param[6])
                .ok_or::<failure::Error>(ManagementError::Unknown.into())?;
            Ok((addr, addr_type))
        })
}

pub fn pin_code_negative_reply(
    sock: &ManagementSocket,
    controller: Controller,
    address: Address,
    address_type: AddressType,
    timeout: i32,
) -> Result<(Address, AddressType), failure::Error> {
    let mut param = vec![];

    param.extend_from_slice(&address.bytes);
    param.push(address_type as u8);

    await_request(
        sock,
        ManagementCommand::PinCodeNegativeReply,
        param,
        controller,
        timeout,
    )?
        .map_success(|param, _, _| {
            let addr = Address::from_slice(&param[0..6]);
            let addr_type: AddressType = FromPrimitive::from_u8(param[6])
                .ok_or::<failure::Error>(ManagementError::Unknown.into())?;
            Ok((addr, addr_type))
        })
}

#[repr(u8)]
pub enum IoCapability {
    DisplayOnly = 0,
    DisplayYesNo,
    KeyboardOnly,
    NoInputNoOutput,
    KeyboardDisplay,
}

pub fn set_io_capability(
    sock: &ManagementSocket,
    controller: Controller,
    io_capability: IoCapability,
    timeout: i32,
) -> Result<(), failure::Error> {
    let mut param = vec![];

    param.push(io_capability as u8);

    await_request(
        sock,
        ManagementCommand::SetIOCapability,
        param,
        controller,
        timeout,
    )?
        .map_success(|_, _, _| Ok(()))
}

pub fn pair_device(
    sock: &ManagementSocket,
    controller: Controller,
    io_capability: IoCapability,
    timeout: i32,
) -> Result<(), failure::Error> {
    let mut param = vec![];

    param.push(io_capability as u8);

    await_request(
        sock,
        ManagementCommand::SetIOCapability,
        param,
        controller,
        timeout,
    )?
        .map_success(|_, _, _| Ok(()))
}
