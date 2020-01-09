use std::ffi::{CString};

use bytes::Bytes;

use crate::Address;
use crate::mgmt::client::{AddressType, DeviceFlags, DisconnectionReason};
use crate::mgmt::interface::{ManagementCommand, ManagementCommandStatus};
use crate::mgmt::interface::controller::ControllerSettings;
use crate::mgmt::interface::class::{ServiceClasses, DeviceClass};

#[derive(Debug)]
pub enum ManagementEvent {
    /// This event is an indication that a command has completed. The
    ///	fixed set of parameters includes the opcode to identify the
    ///	command that completed as well as a status value to indicate
    ///	success or failure. The rest of the parameters are command
    ///	specific and documented in the section for each command
    ///	separately.
    CommandComplete {
        opcode: ManagementCommand,
        status: ManagementCommandStatus,
        param: Bytes,
    },

    /// The command status event is used to indicate an early status for
    ///	a pending command. In the case that the status indicates failure
    ///	(anything else except success status) this also means that the
    ///	command has finished executing.
    CommandStatus {
        opcode: ManagementCommand,
        status: ManagementCommandStatus,
    },

    /// This event maps straight to the HCI Hardware Error event and is
    ///	used to indicate something wrong with the controller hardware.
    ControllerError {
        code: u8,
    },

    /// This event indicates that a new controller has been added to the
    ///	system. It is usually followed by a Read Controller Information
    ///	command.
    IndexAdded,

    /// This event indicates that a controller has been removed from the
    ///	system.
    IndexRemoved,

    /// This event indicates that one or more of the settings for a
    ///	controller has changed.
    NewSettings {
        settings: ControllerSettings,
    },

    /// This event indicates that the Class of Device value for the
    ///	controller has changed. When the controller is powered off the
    ///	Class of Device value will always be reported as zero.
    ClassOfDeviceChanged {
        class: (ServiceClasses, DeviceClass),
    },

    /// This event indicates that the local name of the controller has
    ///	changed.
    LocalNameChanged {
        name: CString,
        short_name: CString,
    },

    /// This event indicates that a new link key has bee generated for a
    ///	remote device. The `store_hint` parameter indicates whether the
    ///	host is expected to store the key persistently or not (e.g. this
    ///	would not be set if the authentication requirement was "No
    ///	Bonding").
    NewLinkKey {
        store_hint: u8,
        address: Address,
        address_type: AddressType,
        key_type: u8,
        value: [u8; 16],
        pin_length: u8,
    },

    /// This event indicates that a new long term key has bee generated
    ///	for a remote device. The `store_hint` parameter indicates whether
    ///	the host is expected to store the key persistently or not (e.g.
    ///	this would not be set if the authentication requirement was "No
    ///	Bonding").
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

    ///	This event indicates that a successful baseband connection has
    ///	been created to the remote device.
    DeviceConnected {
        address: Address,
        address_type: AddressType,
        flags: DeviceFlags,
        eir_data: Bytes,
    },

    /// This event indicates that the baseband connection was lost to a
    ///	remote device.
    ///
    ///	Note that the local/remote distinction just determines which side
    ///	terminated the low-level connection, regardless of the
    ///	disconnection of the higher-level profiles.
    ///
    ///	This can sometimes be misleading and thus must be used with care.
    ///	For example, some hardware combinations would report a locally
    ///	initiated disconnection even if the user turned Bluetooth off in
    ///	the remote side.
    DeviceDisconnected {
        address: Address,
        address_type: AddressType,
        reason: DisconnectionReason,
    },

    /// This event indicates that a connection attempt failed to a
    ///	remote device.
    ConnectFailed {
        address: Address,
        address_type: AddressType,
        status: u8,
    },

    /// This event is used to request a PIN Code reply from user space.
    ///	The reply should either be returned using the PIN Code Reply or
    ///	the PIN Code Negative Reply command. If `secure` is true, then
    /// a secure pin code is required.
    PinCodeRequest {
        address: Address,
        address_type: AddressType,
        secure: bool,
    },

    /// This event is used to request a user confirmation request from
    ///	user space. If `confirm_hint` is true this
    ///	means that a simple "Yes/No" confirmation should be presented to
    ///	the user instead of a full numerical confirmation (in which case
    ///	the parameter value will be false).
    ///
    ///	User space should respond to this command either using the User
    ///	Confirmation Reply or the User Confirmation Negative Reply
    ///	command.
    UserConfirmationRequest {
        address: Address,
        address_type: AddressType,
        confirm_hint: bool,
        value: u32,
    },

    /// This event is used to request a passkey from user space. The
    ///	response to this event should either be the User Passkey Reply
    ///	command or the User Passkey Negative Reply command.
    UserPasskeyRequest {
        address: Address,
        address_type: AddressType,
    },

    /// This event indicates that there was an authentication failure
    ///	with a remote device.
    AuthenticationFailed {
        address: Address,
        address_type: AddressType,
        status: u8,
    },

    /// This event indicates that a device was found during device
    ///	discovery.
    ///
    ///	The Confirm name flag indicates that the kernel wants to know
    ///	whether user space knows the name for this device or not. If
    ///	this flag is set user space should respond to it using the
    ///	Confirm Name command.
    ///
    ///	The Legacy Pairing flag indicates that Legacy Pairing is likely
    ///	to occur when pairing with this device. An application could use
    ///	this information to optimize the pairing process by locally
    ///	pre-generating a PIN code and thereby eliminate the risk of
    ///	local input timeout when pairing. Note that there is a risk of
    ///	false-positives for this flag so user space should be able to
    ///	handle getting something else as a PIN Request when pairing.
    DeviceFound {
        address: Address,
        address_type: AddressType,
        rssi: u8,
        flags: DeviceFlags,
        eir_data: Bytes,
    },

    /// This event indicates that the controller has started discovering
    ///	devices. This discovering state can come and go multiple times
    ///	between a StartDiscover and a StopDiscovery command.
    Discovering {
        address_type: AddressType,
        discovering: bool,
    },

    /// This event indicates that a device has been blocked using the
    ///	Block Device command. The event will only be sent to Management
    ///	sockets other than the one through which the command was sent.
    DeviceBlocked {
        address: Address,
        address_type: AddressType,
    },

    /// This event indicates that a device has been unblocked using the
    ///	Unblock Device command. The event will only be sent to
    ///	Management sockets other than the one through which the command
    ///	was sent.
    DeviceUnblocked {
        address: Address,
        address_type: AddressType,
    },

    /// This event indicates that a device has been unpaired (i.e. all
    ///	its keys have been removed from the kernel) using the Unpair
    ///	Device command. The event will only be sent to Management
    ///	sockets other than the one through which the Unpair Device
    ///	command was sent.
    DeviceUnpaired {
        address: Address,
        address_type: AddressType,
    },

    /// This event is used to request passkey notification to the user.
    ///	Unlike the other authentication events it does not require any response.
    ///
    ///	The `passkey` parameter indicates the passkey to be shown to the
    ///	user. The `entered` parameter indicates how many characters
    ///	the user has entered on the remote side.
    PasskeyNotify {
        address: Address,
        address_type: AddressType,
        passkey: u32,
        entered: u8,
    },
}
