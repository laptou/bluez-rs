use super::*;
use crate::util::BufExt;
use crate::AddressType;

#[inline]
pub(crate) fn get_address(param: Option<Bytes>) -> Result<(Address, AddressType)> {
    let mut param = param.ok_or(Error::NoData)?;
    Ok((param.get_address(), param.get_primitive_u8()))
}

pub(crate) fn address_bytes(address: Address, address_type: AddressType) -> Bytes {
    let mut param = BytesMut::with_capacity(7);
    param.put_slice(address.as_ref());
    param.put_u8(address_type as u8);
    param.freeze()
}

pub(crate) fn address_bytes_with_u8(
    address: Address,
    address_type: AddressType,
    extra: u8,
) -> Bytes {
    let mut param = BytesMut::with_capacity(8);
    param.put_slice(address.as_ref());
    param.put_u8(address_type as u8);
    param.put_u8(extra);
    param.freeze()
}

///	This command is only valid during device discovery and is
///	expected for each Device Found event with the Confirm Name
///	flag set.
///
///	The name_known parameter should be set to true if user space
///	knows the name for the device and false if it doesn't. If set to
///	false the kernel will perform a name resolving procedure for the
///	device in question.
///
///	This command can only be used when the controller is powered.
pub async fn confirm_name(
    socket: &mut ManagementStream,
    controller: Controller,
    address: Address,
    address_type: AddressType,
    name_known: bool,
    event_tx: Option<mpsc::Sender<Response>>,
) -> Result<(Address, AddressType)> {
    let (_, param) = exec_command(
        socket,
        Command::ConfirmName,
        controller,
        Some(address_bytes_with_u8(
            address,
            address_type,
            name_known as u8,
        )),
        event_tx,
    )
    .await?;

    get_address(param)
}

/// This command is used to add a device to the list of devices
///	which should be blocked from being connected to the local
///	controller.
///
///	For Low Energy devices, the blocking of a device takes precedence
///	over auto-connection actions provided by Add Device. Blocked
///	devices will not be auto-connected or even reported when found
///	during background scanning. If the controller is connectable
///	direct advertising from blocked devices will also be ignored.
///
///	Connections created from advertising of the controller will
///	be dropped if the device is blocked.
///
///	This command can be used when the controller is not powered.
pub async fn block_device(
    socket: &mut ManagementStream,
    controller: Controller,
    address: Address,
    address_type: AddressType,
    event_tx: Option<mpsc::Sender<Response>>,
) -> Result<(Address, AddressType)> {
    let (_, param) = exec_command(
        socket,
        Command::BlockDevice,
        controller,
        Some(address_bytes(address, address_type)),
        event_tx,
    )
    .await?;

    get_address(param)
}

/// This command is used to remove a device from the list of blocked
///	devices (where it was added to using the Block Device command).
///
///	When the `address` parameter is `00:00:00:00:00:00`, then all
///	previously blocked devices will be unblocked.
///
///	This command can be used when the controller is not powered.
pub async fn unblock_device(
    socket: &mut ManagementStream,
    controller: Controller,
    address: Address,
    address_type: AddressType,
    event_tx: Option<mpsc::Sender<Response>>,
) -> Result<(Address, AddressType)> {
    let (_, param) = exec_command(
        socket,
        Command::UnblockDevice,
        controller,
        Some(address_bytes(address, address_type)),
        event_tx,
    )
    .await?;

    get_address(param)
}

///	This command is used to force the disconnection of a currently
///	connected device.
///
///	This command can only be used when the controller is powered.
pub async fn disconnect(
    socket: &mut ManagementStream,
    controller: Controller,
    address: Address,
    address_type: AddressType,
    event_tx: Option<mpsc::Sender<Response>>,
) -> Result<(Address, AddressType)> {
    let (_, param) = exec_command(
        socket,
        Command::Disconnect,
        controller,
        Some(address_bytes(address, address_type)),
        event_tx,
    )
    .await?;

    get_address(param)
}

///	This command is used to respond to a PIN Code request event.
/// Pin code can be at most 16 bytes. Passing None will send a
/// negative PIN code response.
///	This command can only be used when the controller is powered.
pub async fn pin_code_reply(
    socket: &mut ManagementStream,
    controller: Controller,
    address: Address,
    address_type: AddressType,
    pin_code: Option<Vec<u8>>,
    event_tx: Option<mpsc::Sender<Response>>,
) -> Result<(Address, AddressType)> {
    let mut param;
    let opcode;

    if let Some(pin_code) = pin_code {
        opcode = Command::PinCodeReply;
        param = BytesMut::with_capacity(24);
        param.put_slice(address.as_ref());
        param.put_u8(address_type as u8);
        param.put_u8(pin_code.len() as u8);
        param.put_slice(&pin_code[..]);
        param.resize(24, 0);
    } else {
        opcode = Command::PinCodeNegativeReply;
        param = BytesMut::with_capacity(7);
        param.put_slice(address.as_ref());
        param.put_u8(address_type as u8);
    }

    let (_, param) =
        exec_command(socket, opcode, controller, Some(param.freeze()), event_tx).await?;

    get_address(param)
}

///	This command is used to trigger pairing with a remote device.
///	The IO_Capability command parameter is used to temporarily (for
///	this pairing event only) override the global IO Capability (set
///	using the Set IO Capability command).
///
///	Passing a value 4 (KeyboardDisplay) will cause the kernel to
///	convert it to 1 (DisplayYesNo) in the case of a BR/EDR
///	connection (as KeyboardDisplay is specific to SMP).
///
///	The `address` and `address_type` of the return parameters will
///	return the identity address if known. In case of resolvable
///	random address given as command parameters and the remote
///	provides an identity resolving key, the return parameters
///	will provide the resolved address.
///
///	To allow tracking of which resolvable random address changed
///	into which identity address, the New Identity Resolving Key
///	event will be sent before receiving Command Complete event
///	for this command.
///
///	This command can only be used when the controller is powered.
pub async fn pair_device(
    socket: &mut ManagementStream,
    controller: Controller,
    address: Address,
    address_type: AddressType,
    io_capability: IoCapability,
    event_tx: Option<mpsc::Sender<Response>>,
) -> Result<(Address, AddressType)> {
    let (_, param) = exec_command(
        socket,
        Command::PairDevice,
        controller,
        Some(address_bytes_with_u8(
            address,
            address_type,
            io_capability as u8,
        )),
        event_tx,
    )
    .await?;

    get_address(param)
}

///	The `address` and `address_type` parameters should match what was
///	given to a preceding Pair Device command.
///
///	This command can only be used when the controller is powered.
pub async fn cancel_pair_device(
    socket: &mut ManagementStream,
    controller: Controller,
    address: Address,
    address_type: AddressType,
    event_tx: Option<mpsc::Sender<Response>>,
) -> Result<(Address, AddressType)> {
    let (_, param) = exec_command(
        socket,
        Command::CancelPairDevice,
        controller,
        Some(address_bytes(address, address_type)),
        event_tx,
    )
    .await?;

    get_address(param)
}

///	Removes all keys associated with the remote device.
///
///	The disconnect parameter tells the kernel whether to forcefully
///	disconnect any existing connections to the device. It should in
///	practice always be true except for some special GAP qualification
///	test-cases where a key removal without disconnecting is needed.
///
///	When unpairing a device its link key, long term key and if
///	provided identity resolving key will be purged.
///
///	For devices using resolvable random addresses where the identity
///	resolving key was available, after this command they will now no
///	longer be resolved. The device will essentially become private
///	again.
///
///	This command can only be used when the controller is powered.
pub async fn unpair_device(
    socket: &mut ManagementStream,
    controller: Controller,
    address: Address,
    address_type: AddressType,
    disconnect: bool,
    event_tx: Option<mpsc::Sender<Response>>,
) -> Result<(Address, AddressType)> {
    let (_, param) = exec_command(
        socket,
        Command::UnpairDevice,
        controller,
        Some(address_bytes_with_u8(
            address,
            address_type,
            disconnect as u8,
        )),
        event_tx,
    )
    .await?;

    get_address(param)
}

///	This command is used to respond to a User Confirmation Request
///	event. This command can only be used when the controller is powered.
pub async fn user_confirmation_reply(
    socket: &mut ManagementStream,
    controller: Controller,
    address: Address,
    address_type: AddressType,
    reply: bool,
    event_tx: Option<mpsc::Sender<Response>>,
) -> Result<(Address, AddressType)> {
    let (_, param) = exec_command(
        socket,
        if reply {
            Command::UserConfirmationReply
        } else {
            Command::UserConfirmationNegativeReply
        },
        controller,
        Some(address_bytes(address, address_type)),
        event_tx,
    )
    .await?;

    get_address(param)
}

///	This command is used to respond to a User Passkey Request
///	event. Passing None for passkey will send a negative response.
/// This command can only be used when the controller is powered.
pub async fn user_passkey_reply(
    socket: &mut ManagementStream,
    controller: Controller,
    address: Address,
    address_type: AddressType,
    passkey: Option<u32>,
    event_tx: Option<mpsc::Sender<Response>>,
) -> Result<(Address, AddressType)> {
    let opcode;
    let mut param;

    if let Some(passkey) = passkey {
        opcode = Command::UserPasskeyReply;
        param = BytesMut::with_capacity(11);
        param.put_slice(address.as_ref());
        param.put_u8(address_type as u8);
        param.put_u32_le(passkey);
    } else {
        opcode = Command::UserPasskeyNegativeReply;
        param = BytesMut::with_capacity(7);
        param.put_slice(address.as_ref());
        param.put_u8(address_type as u8);
    }

    let (_, param) =
        exec_command(socket, opcode, controller, Some(param.freeze()), event_tx).await?;

    get_address(param)
}

///	This command is used to add a device to the action list. The
///	action list allows scanning for devices and enables incoming
///	connections from known devices.
///
///	With the `BackgroundScan` action, when the device is found, a new Device Found
///	event will be sent indicating this device is available. This
///	action is only valid for LE Public and LE Random address types.
///
///	With the `AllowConnect` action, the device is allowed to connect. For BR/EDR
///	address type this means an incoming connection. For LE Public
///	and LE Random address types, a connection will be established
///	to devices using directed advertising. If successful a Device
///	Connected event will be sent.
///
///	With the `AutoConnect`, when the device is found, it will be connected
///	and if successful a Device Connected event will be sent. This
///	action is only valid for LE Public and LE Random address types.
///
///	When a device is blocked using Block Device command, then it is
///	valid to add the device here, but all actions will be ignored
///	until the device is unblocked.
///
///	Devices added with `AllowConnect` are allowed to connect even if the
///	connectable setting is off. This acts as list of known trusted
///	devices.
///
///	This command can be used when the controller is not powered and
///	all settings will be programmed once powered.
pub async fn add_device(
    socket: &mut ManagementStream,
    controller: Controller,
    address: Address,
    address_type: AddressType,
    action: AddDeviceAction,
    event_tx: Option<mpsc::Sender<Response>>,
) -> Result<(Address, AddressType)> {
    let (_, param) = exec_command(
        socket,
        Command::AddDevice,
        controller,
        Some(address_bytes_with_u8(address, address_type, action as u8)),
        event_tx,
    )
    .await?;

    get_address(param)
}

///	This command is used to remove a device from the action list
///	previously added by using the Add Device command.
///
///	When the Address parameter is `00:00:00:00:00:00`, then all
///	previously added devices will be removed.
///
///	This command can be used when the controller is not powered and
///	all settings will be programmed once powered.
pub async fn remove_device(
    socket: &mut ManagementStream,
    controller: Controller,
    address: Address,
    address_type: AddressType,
    event_tx: Option<mpsc::Sender<Response>>,
) -> Result<(Address, AddressType)> {
    let (_, param) = exec_command(
        socket,
        Command::RemoveDevice,
        controller,
        Some(address_bytes(address, address_type)),
        event_tx,
    )
    .await?;

    get_address(param)
}
