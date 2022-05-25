use bytes::{Buf, BufMut, BytesMut};
use enumflags2::BitFlags;

use crate::management::interface::Command;
use crate::management::interface::{Controller, ControllerSettings};
use crate::management::Result;
use crate::Address;

use super::*;
use crate::util::BufExt;

/// This command is used to set the local name of a controller. The
///	command parameters also include a short name which will be used
///	in case the full name doesn't fit within EIR/AD data.
///
/// Name can be at most 248 bytes. Short name can be at most 10 bytes.
/// This function returns a pair of OsStrings in the order (name, short_name).
///
///	This command can be used when the controller is not powered and
///	all settings will be programmed once powered.
///
///	The values of name and short name will be remembered when
///	switching the controller off and back on again. So the name
///	and short name only have to be set once when a new controller
///	is found and will stay until removed.
pub async fn set_local_name(
    socket: &mut ManagementStream,
    controller: Controller,
    name: &str,
    short_name: Option<&str>,
    event_tx: Option<mpsc::Sender<Response>>,
) -> Result<(CString, CString)> {
    if name.len() > 248 {
        return Err(Error::NameTooLong {
            name: name.to_owned(),
            max_len: 248,
        });
    }

    if let Some(short_name) = short_name {
        if short_name.len() > 10 {
            return Err(Error::NameTooLong {
                name: short_name.to_owned(),
                max_len: 10,
            });
        }
    }
    let short_name = short_name.unwrap_or("");

    let mut param = BytesMut::with_capacity(260);
    param.resize(260, 0); // initialize w/ zeros

    CString::new(name)?
        .as_bytes_with_nul()
        .copy_to_slice(&mut param[..=name.len()]);
    CString::new(short_name)?
        .as_bytes_with_nul()
        .copy_to_slice(&mut param[249..][..=short_name.len()]);

    let (_, param) = exec_command(
        socket,
        Command::SetLocalName,
        controller,
        Some(param.freeze()),
        event_tx,
    )
    .await?;

    let mut param = param.ok_or(Error::NoData)?;
    Ok((param.split_to(249).get_c_string(), param.get_c_string()))
}

/// This command is used to power on or off a controller.
///
///	If discoverable setting is activated with a timeout, then
///	switching the controller off will expire this timeout and
///	disable discoverable.
///
///	Settings programmed via Set Advertising and Add/Remove
///	Advertising while the controller was powered off will be activated
///	when powering the controller on.
///
///	Switching the controller off will permanently cancel and remove
///	all advertising instances with a timeout set, i.e. time limited
///	advertising instances are not being remembered across power cycles.
///	Advertising Removed events will be issued accordingly.
pub async fn set_powered(
    socket: &mut ManagementStream,
    controller: Controller,
    powered: bool,
    event_tx: Option<mpsc::Sender<Response>>,
) -> Result<ControllerSettings> {
    let mut param = BytesMut::with_capacity(1);
    param.put_u8(powered as u8);

    let (_, param) = exec_command(
        socket,
        Command::SetPowered,
        controller,
        Some(param.freeze()),
        event_tx,
    )
    .await?;

    Ok(param.ok_or(Error::NoData)?.get_flags_u32_le())
}

/// This command is used to set the discoverable property of a
///	controller.
///
///	Timeout is the time in seconds and is only meaningful when
///	Discoverable is set to General or Limited. Providing a timeout
///	with None returns Invalid Parameters. For Limited, the timeout
///	is required.
///
///	This command is only available for BR/EDR capable controllers
///	(e.g. not for single-mode LE ones). It will return Not Supported
///	otherwise.
///
///	This command can be used when the controller is not powered and
///	all settings will be programmed once powered, however using a timeout
/// when the controller is not powered will	return Not Powered error.
///
///	When switching discoverable on and the connectable setting is
///	off it will return Rejected error.
pub async fn set_discoverable(
    socket: &mut ManagementStream,
    controller: Controller,
    discoverability: DiscoverableMode,
    timeout: Option<u16>,
    event_tx: Option<mpsc::Sender<Response>>,
) -> Result<ControllerSettings> {
    let mut param = BytesMut::with_capacity(3);
    param.put_u8(discoverability as u8);
    if let Some(timeout) = timeout {
        param.put_u16_le(timeout);
    }

    let (_, param) = exec_command(
        socket,
        Command::SetDiscoverable,
        controller,
        Some(param.freeze()),
        event_tx,
    )
    .await?;

    Ok(param.ok_or(Error::NoData)?.get_flags_u32_le())
}

/// This command is used to set the connectable property of a
///	controller.
///
///	This command is available for BR/EDR, LE-only and also dual
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
pub async fn set_connectable(
    socket: &mut ManagementStream,
    controller: Controller,
    connectable: bool,
    event_tx: Option<mpsc::Sender<Response>>,
) -> Result<ControllerSettings> {
    let mut param = BytesMut::with_capacity(1);
    param.put_u8(connectable as u8);

    let (_, param) = exec_command(
        socket,
        Command::SetConnectable,
        controller,
        Some(param.freeze()),
        event_tx,
    )
    .await?;

    Ok(param.ok_or(Error::NoData)?.get_flags_u32_le())
}

/// This command is used to set the controller into a connectable
///	state where the page scan parameters have been set in a way to
///	favor faster connect times with the expense of higher power
///	consumption.
///
///	This command is only available for BR/EDR capable controllers
///	(e.g. not for single-mode LE ones). It will return Not Supported
///	otherwise.
///
///	This command can be used when the controller is not powered and
///	all settings will be programmed once powered.
///
///	The setting will be remembered during power down/up toggles.
pub async fn set_fast_connectable(
    socket: &mut ManagementStream,
    controller: Controller,
    fast_connectable: bool,
    event_tx: Option<mpsc::Sender<Response>>,
) -> Result<ControllerSettings> {
    let mut param = BytesMut::with_capacity(1);
    param.put_u8(fast_connectable as u8);

    let (_, param) = exec_command(
        socket,
        Command::SetFastConnectable,
        controller,
        Some(param.freeze()),
        event_tx,
    )
    .await?;

    Ok(param.ok_or(Error::NoData)?.get_flags_u32_le())
}

/// This command is used to set the bondable (pairable) property of an
///	controller.
///
///	This command can be used when the controller is not powered and
///	all settings will be programmed once powered.
///
///	Turning bondable on will not automatically switch the controller
///	into connectable mode. That needs to be done separately.
///
///	The setting will be remembered during power down/up toggles.
pub async fn set_bondable(
    socket: &mut ManagementStream,
    controller: Controller,
    bondable: bool,
    event_tx: Option<mpsc::Sender<Response>>,
) -> Result<ControllerSettings> {
    let mut param = BytesMut::with_capacity(1);
    param.put_u8(bondable as u8);

    let (_, param) = exec_command(
        socket,
        Command::SetPairable,
        controller,
        Some(param.freeze()),
        event_tx,
    )
    .await?;

    Ok(param.ok_or(Error::NoData)?.get_flags_u32_le())
}

///	This command is used to either enable or disable link level
///	security for an controller (also known as Security Mode 3).
///
///	This command is only available for BR/EDR capable controllers
///	(e.g. not for single-mode LE ones). It will return Not Supported
///	otherwise.
///
///	This command can be used when the controller is not powered and
///	all settings will be programmed once powered.
pub async fn set_link_security(
    socket: &mut ManagementStream,
    controller: Controller,
    link_security: bool,
    event_tx: Option<mpsc::Sender<Response>>,
) -> Result<ControllerSettings> {
    let mut param = BytesMut::with_capacity(1);
    param.put_u8(link_security as u8);

    let (_, param) = exec_command(
        socket,
        Command::SetLinkSecurity,
        controller,
        Some(param.freeze()),
        event_tx,
    )
    .await?;

    Ok(param.ok_or(Error::NoData)?.get_flags_u32_le())
}

///	This command is used to enable/disable Secure Simple Pairing
///	support for a controller.
///
///	This command is only available for BR/EDR capable controllers
///	supporting the core specification version 2.1 or greater
///	(e.g. not for single-mode LE controllers or pre-2.1 ones).
///
///	This command can be used when the controller is not powered and
///	all settings will be programmed once powered.
///
///	In case the controller does not support Secure Simple Pairing,
///	the command will fail regardless with Not Supported error.
pub async fn set_ssp(
    socket: &mut ManagementStream,
    controller: Controller,
    ssp: bool,
    event_tx: Option<mpsc::Sender<Response>>,
) -> Result<ControllerSettings> {
    let mut param = BytesMut::with_capacity(1);
    param.put_u8(ssp as u8);

    let (_, param) = exec_command(
        socket,
        Command::SetSecureSimplePairing,
        controller,
        Some(param.freeze()),
        event_tx,
    )
    .await?;

    Ok(param.ok_or(Error::NoData)?.get_flags_u32_le())
}

///	This command is used to enable/disable Bluetooth High Speed
///	support for a controller.
///
///	This command is only available for BR/EDR capable controllers
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
pub async fn set_high_speed(
    socket: &mut ManagementStream,
    controller: Controller,
    high_speed: bool,
    event_tx: Option<mpsc::Sender<Response>>,
) -> Result<ControllerSettings> {
    let mut param = BytesMut::with_capacity(1);
    param.put_u8(high_speed as u8);

    let (_, param) = exec_command(
        socket,
        Command::SetHighSpeed,
        controller,
        Some(param.freeze()),
        event_tx,
    )
    .await?;

    Ok(param.ok_or(Error::NoData)?.get_flags_u32_le())
}

/// This command is used to enable/disable Low Energy support for a
///	controller.
///
///	This command is only available for LE capable controllers and
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
pub async fn set_le(
    socket: &mut ManagementStream,
    controller: Controller,
    le: bool,
    event_tx: Option<mpsc::Sender<Response>>,
) -> Result<ControllerSettings> {
    let mut param = BytesMut::with_capacity(1);
    param.put_u8(le as u8);

    let (_, param) = exec_command(
        socket,
        Command::SetLowEnergy,
        controller,
        Some(param.freeze()),
        event_tx,
    )
    .await?;

    Ok(param.ok_or(Error::NoData)?.get_flags_u32_le())
}

/// This command is used to enable LE advertising on a controller
///	that supports it.
///
///	The value `Disabled` disables advertising, the value `WithConnectable` enables
///	advertising with considering of connectable setting and the
///	value `Enabled` enables advertising in connectable mode.
///
///	Using value `WithConnectable` means that when connectable setting is disabled,
///	the advertising happens with undirected non-connectable advertising
///	packets and a non-resolvable random address is used. If connectable
///	setting is enabled, then undirected connectable advertising packets
///	and the identity address or resolvable private address are used.
///
///	LE Devices configured via Add Device command with Action `0x01`
///	have no effect when using Advertising value `0x01` since only the
///	connectable setting is taken into account.
///
///	To utilize undirected connectable advertising without changing the
///	connectable setting, the value `Enabled` can be utilized. It makes the
///	device connectable via LE without the requirement for being
///	connectable on BR/EDR (and/or LE).
///
///	The value `Enabled` should be the preferred mode of operation when
///	implementing peripheral mode.
///
///	Using this command will temporarily deactivate any configuration
///	made by the Add Advertising command. This command takes precedence.
///	Once a Set Advertising command with value `Disabled` is issued any
///	previously made configurations via Add/Remove Advertising, including
///	such changes made while Set Advertising was active, will be re-
///	enabled.
///
///	A pre-requisite is that LE is already enabled, otherwise this
///	command will return a "rejected" response.
pub async fn set_advertising(
    socket: &mut ManagementStream,
    controller: Controller,
    mode: LeAdvertisingMode,
    event_tx: Option<mpsc::Sender<Response>>,
) -> Result<ControllerSettings> {
    let mut param = BytesMut::with_capacity(1);
    param.put_u8(mode as u8);

    let (_, param) = exec_command(
        socket,
        Command::SetAdvertising,
        controller,
        Some(param.freeze()),
        event_tx,
    )
    .await?;

    Ok(param.ok_or(Error::NoData)?.get_flags_u32_le())
}

/// This command is used to enable or disable BR/EDR support
///	on a dual-mode controller.
///
///	A pre-requisite is that LE is already enabled, otherwise
///	this command will return a "rejected" response. Enabling BR/EDR
///	can be done both when powered on and powered off, however
///	disabling it can only be done when powered off (otherwise the
///	command will again return "rejected"). Disabling BR/EDR will
///	automatically disable all other BR/EDR related settings.
pub async fn set_bredr(
    socket: &mut ManagementStream,
    controller: Controller,
    enabled: bool,
    event_tx: Option<mpsc::Sender<Response>>,
) -> Result<ControllerSettings> {
    let mut param = BytesMut::with_capacity(1);
    param.put_u8(enabled as u8);

    let (_, param) = exec_command(
        socket,
        Command::SetBREDR,
        controller,
        Some(param.freeze()),
        event_tx,
    )
    .await?;

    Ok(param.ok_or(Error::NoData)?.get_flags_u32_le())
}

///	This command is used to set the IO Capability used for pairing.
///	The command accepts both SSP and SMP values.
///
///	Passing KeyboardDisplay will cause the kernel to
///	convert it to DisplayYesNo)in the case of a BR/EDR
///	connection (as KeyboardDisplay is specific to SMP).
///
///	This command can be used when the controller is not powered.
pub async fn set_io_capability(
    socket: &mut ManagementStream,
    controller: Controller,
    io_capability: IoCapability,
    event_tx: Option<mpsc::Sender<Response>>,
) -> Result<()> {
    let mut param = BytesMut::with_capacity(1);
    param.put_u8(io_capability as u8);

    let (_, _param) = exec_command(
        socket,
        Command::SetIOCapability,
        controller,
        Some(param.freeze()),
        event_tx,
    )
    .await?;

    Ok(())
}

/// This command can be used when the controller is not powered and
///	all settings will be programmed once powered.
///
///	The Source parameter selects the organization that assigned the
///	Vendor parameter:
///
///	 - `0x0000`	Disable Device ID
///	 - `0x0001`	Bluetooth SIG
///	 - `0x0002`	USB Implementer's Forum
///
///	The information is put into the EIR data. If the controller does
///	not support EIR or if SSP is disabled, this command will still
///	succeed. The information is stored for later use and will survive
///	toggling SSP on and off.
pub async fn set_device_id(
    socket: &mut ManagementStream,
    controller: Controller,
    source: u16,
    vendor: u16,
    product: u16,
    version: u16,
    event_tx: Option<mpsc::Sender<Response>>,
) -> Result<()> {
    let mut param = BytesMut::with_capacity(8);
    param.put_u16_le(source);
    param.put_u16_le(vendor);
    param.put_u16_le(product);
    param.put_u16_le(version);

    let (_, _param) = exec_command(
        socket,
        Command::SetDeviceID,
        controller,
        Some(param.freeze()),
        event_tx,
    )
    .await?;

    Ok(())
}

/// This command allows for setting the Low Energy scan parameters
///	used for connection establishment and passive scanning. It is
///	only supported on controllers with LE support.
pub async fn set_scan_parameters(
    socket: &mut ManagementStream,
    controller: Controller,
    interval: u16,
    window: u16,
    event_tx: Option<mpsc::Sender<Response>>,
) -> Result<()> {
    let mut param = BytesMut::with_capacity(4);
    param.put_u16_le(interval);
    param.put_u16_le(window);

    let (_, _param) = exec_command(
        socket,
        Command::SetScanParameters,
        controller,
        Some(param.freeze()),
        event_tx,
    )
    .await?;

    Ok(())
}

///	This command allows for setting the static random address. It is
///	only supported on controllers with LE support. The static random
///	address is suppose to be valid for the lifetime of the
///	controller or at least until the next power cycle. To ensure
///	such behavior, setting of the address is limited to when the
///	controller is powered off.
///
///	The `Address::zero()` (`00:00:00:00:00:00`) can be used
///	to disable the static address.
///
///	When a controller has a public address (which is required for
///	all dual-mode controllers), this address is not used. If a dual-mode
///	controller is configured as Low Energy only devices (BR/EDR has
///	been switched off), then the static address is used. Only when
///	the controller information reports a zero address (`00:00:00:00:00:00`),
///	it is required to configure a static address first.
///
///	If privacy mode is enabled and the controller is single mode
///	LE only without a public address, the static random address is
///	used as identity address.
///
///	The Static Address flag from the current settings can also be used
///	to determine if the configured static address is in use or not.
pub async fn set_static_address(
    socket: &mut ManagementStream,
    controller: Controller,
    address: Address,
    event_tx: Option<mpsc::Sender<Response>>,
) -> Result<ControllerSettings> {
    let param = BytesMut::from(address.as_ref());

    let (_, param) = exec_command(
        socket,
        Command::SetStaticAddress,
        controller,
        Some(param.freeze()),
        event_tx,
    )
    .await?;

    Ok(param.ok_or(Error::NoData)?.get_flags_u32_le())
}

///	This command is used to enable/disable Secure Connections
///	support for a controller.
///
///	The value `Disabled` disables Secure Connections, the value `Enabled`
///	enables Secure Connections and the value `Only` enables Secure
///	Connections Only mode.
///
///	This command is only available for LE capable controllers as
///	well as controllers supporting the core specification version
///	4.1 or greater.
///
///	This command can be used when the controller is not powered and
///	all settings will be programmed once powered.
///
///	In case the controller does not support Secure Connections
///	the command will fail regardless with Not Supported error.
pub async fn set_secure_connections_mode(
    socket: &mut ManagementStream,
    controller: Controller,
    mode: SecureConnectionsMode,
    event_tx: Option<mpsc::Sender<Response>>,
) -> Result<ControllerSettings> {
    let mut param = BytesMut::with_capacity(1);
    param.put_u8(mode as u8);

    let (_, param) = exec_command(
        socket,
        Command::SetSecureConnections,
        controller,
        Some(param.freeze()),
        event_tx,
    )
    .await?;

    Ok(param.ok_or(Error::NoData)?.get_flags_u32_le())
}

/// This command is used to tell the kernel whether to accept the
///	usage of debug keys or not.
///
///	With a value of `Discard` any generated debug key will be discarded
///	as soon as the connection terminates.
///
///	With a value of `Persist` generated debug keys will be kept and can
///	be used for future connections. However debug keys are always
///	marked as non persistent and should not be stored. This means
///	a reboot or changing the value back to `0x00` will delete them.
///
///	With a value of `PersistAndGenerate` generated debug keys will be kept and can
///	be used for future connections. This has the same affect as
///	with value `Persist`. However in addition this value will also
///	enter the controller mode to generate debug keys for each
///	new pairing. Changing the value back to `Persist` or `Discard` will
///	disable the controller mode for generating debug keys.
pub async fn set_debug_mode(
    socket: &mut ManagementStream,
    controller: Controller,
    mode: DebugKeysMode,
    event_tx: Option<mpsc::Sender<Response>>,
) -> Result<ControllerSettings> {
    let mut param = BytesMut::with_capacity(1);
    param.put_u8(mode as u8);

    let (_, param) = exec_command(
        socket,
        Command::SetDebugKeys,
        controller,
        Some(param.freeze()),
        event_tx,
    )
    .await?;

    Ok(param.ok_or(Error::NoData)?.get_flags_u32_le())
}

///	This command is used to enable Low Energy Privacy feature using
///	resolvable private addresses.
///
///	The value `Disabled` disables privacy mode, the values `Strict` and `Limited`
///	enable privacy mode.
///
///	With value `Strict` the kernel will always use the privacy mode. This
///	means resolvable private address is used when the controller is
///	discoverable and also when pairing is initiated.
///
///	With value `Limited` the kernel will use a limited privacy mode with a
///	resolvable private address except when the controller is bondable
///	and discoverable, in which case the identity address is used.
///
///	Exposing the identity address when bondable and discoverable or
///	during initiated pairing can be a privacy issue. For dual-mode
///	controllers this can be neglected since its public address will
///	be exposed over BR/EDR anyway. The benefit of exposing the
///	identity address for pairing purposes is that it makes matching
///	up devices with dual-mode topology during device discovery now
///	possible.
///
///	If the privacy value `Limited` is used, then also the GATT database
///	should expose the Privacy Characteristic so that remote devices
///	can determine if the privacy feature is in use or not.
///
///	When the controller has a public address (mandatory for dual-mode
///	controllers) it is used as identity address. In case the controller
///	is single mode LE only without a public address, it is required
///	to configure a static random address first. The privacy mode can
///	only be enabled when an identity address is available.
///
///	The identity_resolving_key is the local key assigned for the local
///	resolvable private address.
pub async fn set_privacy_mode(
    socket: &mut ManagementStream,
    controller: Controller,
    mode: PrivacyMode,
    identity_resolving_key: [u8; 16],
    event_tx: Option<mpsc::Sender<Response>>,
) -> Result<ControllerSettings> {
    let mut param = BytesMut::with_capacity(17);
    param.put_u8(mode as u8);
    param.put_slice(&identity_resolving_key[..]);

    let (_, param) = exec_command(
        socket,
        Command::SetPrivacy,
        controller,
        Some(param.freeze()),
        event_tx,
    )
    .await?;

    Ok(param.ok_or(Error::NoData)?.get_flags_u32_le())
}

///	This command allows to change external configuration option to
///	indicate that a controller is now configured or unconfigured.
///
///	The value false sets unconfigured state and the value true sets
///	configured state of the controller.
///
///	It is not mandatory that this configuration option is provided
///	by a controller. If it is provided, the configuration has to
///	happen externally using user channel operation or via vendor
///	specific methods.
///
///	Setting this option and when Missing_Options returns zero, this
///	means that the controller will switch to configured state and it
///	can be expected that it will be announced via Index Added event.
///
///	Wrongly configured controllers might still cause an error when
///	trying to power them via Set Powered command.
pub async fn set_external_config(
    socket: &mut ManagementStream,
    controller: Controller,
    config: bool,
    event_tx: Option<mpsc::Sender<Response>>,
) -> Result<ControllerSettings> {
    let param = BytesMut::from([config as u8].as_ref() as &[u8]);

    let (_, param) = exec_command(
        socket,
        Command::SetExternalConfig,
        controller,
        Some(param.freeze()),
        event_tx,
    )
    .await?;

    Ok(param.ok_or(Error::NoData)?.get_flags_u32_le())
}

///	This command allows configuration of public address. Since a vendor
///	specific procedure is required, this command might not be supported
///	by all controllers. Actually most likely only a handful embedded
///	controllers will offer support for this command.
///
///	When the support for Bluetooth public address configuration is
///	indicated in the supported options mask, then this command
///	can be used to configure the public address.
///
///	It is only possible to configure the public address when the
///	controller is powered off.
///
///	For an unconfigured controller and when this function returns
///	an empty mask, this means that a Index Added event for the now
///	fully configured controller can be expected.
///
///	For a fully configured controller, the current controller index
///	will become invalid and an Unconfigured Index Removed event will
///	be sent. Once the address has been successfully changed an Index
///	Added event will be sent. There is no guarantee that the controller
///	index stays the same.
///
///	All previous configured parameters and settings are lost when
///	this command succeeds. The controller has to be treated as new
///	one. Use this command for a fully configured controller only when
///	you really know what you are doing.
pub async fn set_public_address(
    socket: &mut ManagementStream,
    controller: Controller,
    address: Address,
    event_tx: Option<mpsc::Sender<Response>>,
) -> Result<ControllerSettings> {
    let param = BytesMut::from(address.as_ref());

    let (_, param) = exec_command(
        socket,
        Command::SetPublicAddress,
        controller,
        Some(param.freeze()),
        event_tx,
    )
    .await?;

    Ok(param.ok_or(Error::NoData)?.get_flags_u32_le())
}

///	This command is used to set the appearance value of a controller.
///
///	This command can be used when the controller is not
///	powered and all settings will be programmed once powered.
///
///	The value of appearance will be remembered when switching
///	the controller off and back on again. So the appearance only
///	have to be set once when a new controller is found and will
///	stay until removed.
// todo: implement appearance as enum instead of u16
pub async fn set_appearance(
    socket: &mut ManagementStream,
    controller: Controller,
    appearance: u16,
    event_tx: Option<mpsc::Sender<Response>>,
) -> Result<()> {
    let mut param = BytesMut::with_capacity(2);
    param.put_u16_le(appearance);

    let (_, _param) = exec_command(
        socket,
        Command::SetAppearance,
        controller,
        Some(param.freeze()),
        event_tx,
    )
    .await?;

    Ok(())
}

///	on the PHY configuration. It is remembered over power cycles.
pub async fn set_phy_config(
    socket: &mut ManagementStream,
    controller: Controller,
    selected_phys: BitFlags<PhyFlag>,
    event_tx: Option<mpsc::Sender<Response>>,
) -> Result<()> {
    let mut param = BytesMut::with_capacity(4);
    param.put_u32_le(selected_phys.bits());

    let (_, _param) = exec_command(
        socket,
        Command::SetPhyConfig,
        controller,
        Some(param.freeze()),
        event_tx,
    )
    .await?;

    Ok(())
}

/// This command is used to enable/disable Wideband Speech
/// support for a controller.
///
/// This command is only available for BR/EDR capable controllers and
/// require controller specific support.
///
/// This command can be used when the controller is not powered and
/// all settings will be programmed once powered.
///
/// In case the controller does not support Wideband Speech
/// the command will fail regardless with Not Supported error.
pub async fn set_wideband_speech(
    socket: &mut ManagementStream,
    controller: Controller,
    enabled: bool,
    event_tx: Option<mpsc::Sender<Response>>,
) -> Result<ControllerSettings> {
    let mut param = BytesMut::with_capacity(1);
    param.put_u8(enabled as u8);

    let (_, param) = exec_command(
        socket,
        Command::SetWidebandSpeech,
        controller,
        Some(param.freeze()),
        event_tx,
    )
    .await?;

    Ok(param.ok_or(Error::NoData)?.get_flags_u32_le())
}

/// This command is used to set a list of default runtime parameters.
///
/// This command can be used at any time and will change the runtime
/// default. Changes however will not apply to existing connections or
/// currently active operations.
///
/// When providing unsupported values or invalid values, no parameter
/// value will be changed and all values discarded.
pub async fn set_default_runtime_config(
    socket: &mut ManagementStream,
    controller: Controller,
    params: &[(RuntimeConfigParameterType, Vec<u8>)],
    event_tx: Option<mpsc::Sender<Response>>,
) -> Result<()> {
    let size = params
        .iter()
        .fold(0, |acc, (_, value)| acc + 3 + value.len());
    let mut param = BytesMut::with_capacity(size);

    #[allow(unreachable_code, unused_variables)]
    // until we have constants in RuntimeConfigParameterType
    for (parameter_type, value) in params {
        param.put_u16_le(unimplemented!("*parameter_type as u16"));
        param.put_u8(value.len() as u8);
        param.put_slice(value);
    }

    let (_, _param) = exec_command(
        socket,
        Command::SetDefaultSystemConfig,
        controller,
        Some(param.freeze()),
        event_tx,
    )
    .await?;

    Ok(())
}

/// This command is used to set a list of default controller parameters.
///
/// This command can be used when the controller is not powered and
/// all supported parameters will be programmed once powered.
///
/// When providing unsupported values or invalid values, no parameter
/// value will be changed and all values discarded.
pub async fn set_default_system_config(
    socket: &mut ManagementStream,
    controller: Controller,
    params: &[(SystemConfigParameterType, Vec<u8>)],
    event_tx: Option<mpsc::Sender<Response>>,
) -> Result<()> {
    let size = params
        .iter()
        .fold(0, |acc, (_, value)| acc + 3 + value.len());
    let mut param = BytesMut::with_capacity(size);

    for (parameter_type, value) in params {
        param.put_u16_le(*parameter_type as u16);
        param.put_u8(value.len() as u8);
        param.put_slice(value);
    }

    let (_, _param) = exec_command(
        socket,
        Command::SetDefaultSystemConfig,
        controller,
        Some(param.freeze()),
        event_tx,
    )
    .await?;

    Ok(())
}
