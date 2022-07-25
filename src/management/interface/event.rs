use std::ffi::CString;

use bytes::Bytes;
use enumflags2::BitFlags;

use crate::address::AddressType;
use crate::management::client::*;
use crate::management::interface::class::{DeviceClass, ServiceClasses};
use crate::management::interface::controller::ControllerSettings;
use crate::management::interface::{Command, CommandStatus};
use crate::Address;
use std::collections::HashMap;

#[derive(Debug)]
pub enum Event {
    /// This event is an indication that a command has completed. The
    /// fixed set of parameters includes the opcode to identify the
    /// command that completed as well as a status value to indicate
    /// success or failure. The rest of the parameters are command
    /// specific and documented in the section for each command
    /// separately.
    CommandComplete {
        opcode: Command,
        status: CommandStatus,
        param: Bytes,
    },

    /// The command status event is used to indicate an early status for
    /// a pending command. In the case that the status indicates failure
    /// (anything else except success status) this also means that the
    /// command has finished executing.
    CommandStatus {
        opcode: Command,
        status: CommandStatus,
    },

    /// This event maps straight to the HCI Hardware Error event and is
    /// used to indicate something wrong with the controller hardware.
    ControllerError { code: u8 },

    /// This event indicates that a new controller has been added to the
    /// system. It is usually followed by a Read Controller Information
    /// command.
    IndexAdded,

    /// This event indicates that a controller has been removed from the
    /// system.
    IndexRemoved,

    /// This event indicates that one or more of the settings for a
    /// controller has changed.
    NewSettings { settings: ControllerSettings },

    /// This event indicates that the Class of Device value for the
    /// controller has changed. When the controller is powered off the
    /// Class of Device value will always be reported as zero.
    ClassOfDeviceChanged {
        class: (DeviceClass, ServiceClasses),
    },

    /// This event indicates that the local name of the controller has
    /// changed.
    LocalNameChanged { name: CString, short_name: CString },

    /// This event indicates that a new link key has bee generated for a
    /// remote device. The `store_hint` parameter indicates whether the
    /// host is expected to store the key persistently or not (e.g. this
    /// would not be set if the authentication requirement was "No
    /// Bonding").
    NewLinkKey {
        store_hint: bool,
        address: Address,
        address_type: AddressType,
        key_type: LinkKeyType,
        value: [u8; 16],
        pin_length: u8,
    },

    /// This event indicates that a new long term key has bee generated
    /// for a remote device. The `store_hint` parameter indicates whether
    /// the host is expected to store the key persistently or not (e.g.
    /// this would not be set if the authentication requirement was "No
    /// Bonding").
    NewLongTermKey {
        store_hint: bool,
        address: Address,
        address_type: AddressType,
        key_type: LongTermKeyType,
        master: u8,
        encryption_size: u8,
        encryption_diversifier: u16,
        random_number: u64,
        value: [u8; 16],
    },

    /// This event indicates that a successful baseband connection has
    /// been created to the remote device.
    DeviceConnected {
        address: Address,
        address_type: AddressType,
        flags: BitFlags<DeviceFlag>,
        eir_data: Bytes,
    },

    /// This event indicates that the baseband connection was lost to a
    /// remote device.
    ///
    /// Note that the local/remote distinction just determines which side
    /// terminated the low-level connection, regardless of the
    /// disconnection of the higher-level profiles.
    ///
    /// This can sometimes be misleading and thus must be used with care.
    /// For example, some hardware combinations would report a locally
    /// initiated disconnection even if the user turned Bluetooth off in
    /// the remote side.
    DeviceDisconnected {
        address: Address,
        address_type: AddressType,
        reason: DisconnectionReason,
    },

    /// This event indicates that a connection attempt failed to a
    /// remote device.
    ConnectFailed {
        address: Address,
        address_type: AddressType,
        status: u8,
    },

    /// This event is used to request a PIN Code reply from user space.
    /// The reply should either be returned using the PIN Code Reply or
    /// the PIN Code Negative Reply command. If `secure` is true, then
    /// a secure pin code is required.
    PinCodeRequest {
        address: Address,
        address_type: AddressType,
        secure: bool,
    },

    /// This event is used to request a user confirmation request from
    /// user space. If `confirm_hint` is true this
    /// means that a simple "Yes/No" confirmation should be presented to
    /// the user instead of a full numerical confirmation (in which case
    /// the parameter value will be false).
    ///
    /// User space should respond to this command either using the User
    /// Confirmation Reply or the User Confirmation Negative Reply
    /// command.
    UserConfirmationRequest {
        address: Address,
        address_type: AddressType,
        confirm_hint: bool,
        value: u32,
    },

    /// This event is used to request a passkey from user space. The
    /// response to this event should either be the User Passkey Reply
    /// command or the User Passkey Negative Reply command.
    UserPasskeyRequest {
        address: Address,
        address_type: AddressType,
    },

    /// This event indicates that there was an authentication failure
    /// with a remote device.
    AuthenticationFailed {
        address: Address,
        address_type: AddressType,
        status: u8,
    },

    /// This event indicates that a device was found during device
    /// discovery.
    ///
    /// The Confirm name flag indicates that the kernel wants to know
    /// whether user space knows the name for this device or not. If
    /// this flag is set user space should respond to it using the
    /// Confirm Name command.
    ///
    /// The Legacy Pairing flag indicates that Legacy Pairing is likely
    /// to occur when pairing with this device. An application could use
    /// this information to optimize the pairing process by locally
    /// pre-generating a PIN code and thereby eliminate the risk of
    /// local input timeout when pairing. Note that there is a risk of
    /// false-positives for this flag so user space should be able to
    /// handle getting something else as a PIN Request when pairing.
    DeviceFound {
        address: Address,
        address_type: AddressType,
        rssi: i8,
        flags: BitFlags<DeviceFlag>,
        eir_data: Bytes,
    },

    /// This event indicates that the controller has started discovering
    /// devices. This discovering state can come and go multiple times
    /// between a StartDiscover and a StopDiscovery command.
    Discovering {
        address_type: BitFlags<AddressTypeFlag>,
        discovering: bool,
    },

    /// This event indicates that a device has been blocked using the
    /// Block Device command. The event will only be sent to Management
    /// sockets other than the one through which the command was sent.
    DeviceBlocked {
        address: Address,
        address_type: AddressType,
    },

    /// This event indicates that a device has been unblocked using the
    /// Unblock Device command. The event will only be sent to
    /// Management sockets other than the one through which the command
    /// was sent.
    DeviceUnblocked {
        address: Address,
        address_type: AddressType,
    },

    /// This event indicates that a device has been unpaired (i.e. all
    /// its keys have been removed from the kernel) using the Unpair
    /// Device command. The event will only be sent to Management
    /// sockets other than the one through which the Unpair Device
    /// command was sent.
    DeviceUnpaired {
        address: Address,
        address_type: AddressType,
    },

    /// This event is used to request passkey notification to the user.
    /// Unlike the other authentication events it does not require any response.
    ///
    /// The `passkey` parameter indicates the passkey to be shown to the
    /// user. The `entered` parameter indicates how many characters
    /// the user has entered on the remote side.
    PasskeyNotify {
        address: Address,
        address_type: AddressType,
        passkey: u32,
        entered: u8,
    },

    /// This event indicates that a new identity resolving key has been
    /// generated for a remote device.
    ///
    /// The `store_hint` parameter indicates whether the host is expected
    /// to store the key persistently or not.
    ///
    /// The `random_address` provides the resolvable random address that
    /// was resolved into an identity. A value of 00:00:00:00:00:00
    /// indicates that the identity resolving key was provided for
    /// a public address or static random address.
    ///
    /// Once this event has been send for a resolvable random address,
    /// all further events mapping this device will send out using the
    /// identity address information.
    ///
    /// This event also indicates that now the identity address should
    /// be used for commands instead of the resolvable random address.
    ///
    /// It is possible that some devices allow discovering via its
    /// identity address, but after pairing using resolvable private
    /// address only. In such a case `store_hint` will be false` and the
    /// `random_address` will indicate `00:00:00:00:00:00`. For these devices,
    /// the Privacy Characteristic of the remote GATT database should
    /// be consulted to decide if the identity resolving key must be
    /// stored persistently or not.
    ///
    /// Devices using Set Privacy command with the option 0x02 would
    /// be such type of device.
    NewIdentityResolvingKey {
        store_hint: bool,
        random_address: Address,
        address: Address,
        address_type: AddressType,
        value: [u8; 16],
    },

    /// This event indicates that a new signature resolving key has been
    /// generated for either the master or slave device.
    ///
    /// The `store_hint` parameter indicates whether the host is expected
    /// to store the key persistently or not.
    ///
    /// The local keys are used for signing data to be sent to the
    /// remote device, whereas the remote keys are used to verify
    /// signatures received from the remote device.
    ///
    /// The local signature resolving key will be generated with each
    /// pairing request. Only after receiving this event with the Type
    /// indicating a local key is it possible to use ATT Signed Write
    /// procedures.
    ///
    /// The provided `address` and `address_type` are the identity of
    /// a device. So either its public address or static random address.
    NewSignatureResolvingKey {
        store_hint: bool,
        address: Address,
        address_type: AddressType,
        key_type: SignatureResolvingKeyType,
        value: [u8; 16],
    },

    /// This event indicates that a device has been added using the
    /// Add Device command.
    ///
    /// The event will only be sent to management sockets other than the
    /// one through which the command was sent.
    DeviceAdded {
        address: Address,
        address_type: AddressType,
        action: AddDeviceAction,
    },

    /// This event indicates that a device has been removed using the
    /// Remove Device command.
    ///
    /// The event will only be sent to management sockets other than the
    /// one through which the command was sent.
    DeviceRemoved {
        address: Address,
        address_type: AddressType,
    },

    /// This event indicates a new set of connection parameters from
    /// a peripheral device.
    ///
    /// The `store_hint` parameter indicates whether the host is expected
    /// to store this information persistently or not.
    ///
    /// The `min_connection_interval`, `max_connection_interval`,
    /// `connection_latency` and `supervision_timeout` parameters are
    /// encoded as described in Core 4.1 spec, Vol 2, 7.7.65.3.
    NewConnectionParams {
        store_hint: bool,
        param: ConnectionParams,
    },

    /// This event indicates that a new unconfigured controller has been
    /// added to the system. It is usually followed by a Read Controller
    /// Configuration Information command.
    ///
    /// Only when a controller requires further configuration, it will
    /// be announced with this event. If it supports configuration, but
    /// does not require it, then an Index Added event will be used.
    ///
    /// Once the Read Extended Controller Index List command has been
    /// used at least once, the Extended Index Added event will be
    /// send instead of this one.
    UnconfiguredIndexAdded,

    /// This event indicates that an unconfigured controller has been
    /// removed from the system.
    ///
    /// Once the Read Extended Controller Index List command has been
    /// used at least once, the Extended Index Removed event will be
    /// send instead of this one.
    UnconfiguredIndexRemoved,

    /// This event indicates that one or more of the options for the
    /// controller configuration has changed.
    NewConfigOptions {
        missing_options: BitFlags<ControllerConfigOptions>,
    },

    /// This event indicates that a new controller index has been
    /// added to the system.
    ///
    /// This event will only be used after Read Extended Controller Index
    /// List has been used at least once. If it has not been used, then
    /// Index Added and Unconfigured Index Added are sent instead.
    ExtendedIndexAdded {
        controller_type: ControllerType,
        controller_bus: ControllerBus,
    },

    /// This event indicates that a new controller index has been
    /// removed from the system.
    ///
    /// This event will only be used after Read Extended Controller Index
    /// List has been used at least once. If it has not been used, then
    /// Index Added and Unconfigured Index Added are sent instead.
    ExtendedIndexRemoved {
        controller_type: ControllerType,
        controller_bus: ControllerBus,
    },

    /// This event is used when the Read Local Out Of Band Extended Data
    /// command has been used and some other user requested a new set
    /// of local out-of-band data. This allows for the original caller
    /// to adjust the data.
    ///
    /// When LE Privacy is used and LE Secure Connections out-of-band
    /// data has been requested, then this event will be emitted every
    /// time the Resolvable Private Address (RPA) gets changed. The new
    /// RPA will be included in the `eir_data`.
    ///
    /// The event will only be sent to management sockets other than the
    /// one through which the command was sent. It will additionally also
    /// only be sent to sockets that have used the command at least once.
    LocalOutOfBandExtDataUpdated {
        address_type: AddressType,
        eir_data: Bytes,
    },

    /// This event indicates that an advertising instance has been added
    /// using the Add Advertising command.
    ///
    /// The event will only be sent to management sockets other than the
    /// one through which the command was sent.
    AdvertisingAdded { instance: u8 },

    /// This event indicates that an advertising instance has been removed
    /// using the Remove Advertising command.
    ///
    /// The event will only be sent to management sockets other than the
    /// one through which the command was sent.
    AdvertisingRemoved { instance: u8 },

    /// This event indicates that controller information has been updated
    /// and new values are used. This includes the local name, class of
    /// device, device id and LE address information.
    ///
    /// This event will only be used after Read Extended Controller
    /// Information command has been used at least once. If it has not
    /// been used the legacy events are used.
    ///
    /// The event will only be sent to management sockets other than the
    /// one through which the change was triggered.
    ExtControllerInfoChanged { eir_data: Bytes },

    /// This event indicates that an advertising instance has been added
    /// using the Add Advertising command.
    ///
    /// The event will only be sent to management sockets other than the
    /// one through which the command was sent.
    PhyConfigChanged { selected_phys: BitFlags<PhyFlag> },

    /// This event indicates that the status of an experimental feature
    /// has been changed.
    ///
    /// The event will only be sent to management sockets other than the
    /// one through which the change was triggered.
    ExperimentalFeatureChanged { uuid: [u8; 16], flags: u32 },

    /// This event indicates the change of default system parameter values.
    ///
    /// The event will only be sent to management sockets other than the
    ///	one through which the change was trigged. In addition it will
    ///	only be sent to sockets that have issues the Read Default System
    ///	Configuration command.
    DefaultSystemConfigChanged {
        params: HashMap<SystemConfigParameterType, Vec<u8>>,
    },

    ///	This event indicates the change of default runtime parameter values.
    ///
    ///	The event will only be sent to management sockets other than the
    ///	one through which the change was trigged. In addition it will
    ///	only be sent to sockets that have issues the Read Default Runtime
    ///	Configuration command.
    DefaultRuntimeConfigChanged {
        params: HashMap<RuntimeConfigParameterType, Vec<u8>>,
    },
}
