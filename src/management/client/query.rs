use crate::AddressType;
use std::collections::HashMap;

use crate::management::interface::ControllerInfoExt;
use crate::util::BufExt;

use super::*;

/// This command returns the Management version and revision.
///	Besides, being informational the information can be used to
///	determine whether certain behavior has changed or bugs fixed
///	when interacting with the kernel.
pub async fn get_mgmt_version(
    socket: &mut ManagementStream,
    event_tx: Option<mpsc::Sender<Response>>,
) -> Result<ManagementVersion> {
    let (_, param) = exec_command(
        socket,
        Command::ReadVersionInfo,
        Controller::none(),
        None,
        event_tx,
    )
    .await?;

    let mut param = param.ok_or(Error::NoData)?;
    Ok(ManagementVersion {
        version: param.get_u8(),
        revision: param.get_u16_le(),
    })
}

/// This command returns the list of currently known controllers.
///	Controllers added or removed after calling this command can be
///	monitored using the Index Added and Index Removed events.
pub async fn get_controller_list(
    socket: &mut ManagementStream,
    event_tx: Option<mpsc::Sender<Response>>,
) -> Result<Vec<Controller>> {
    let (_, param) = exec_command(
        socket,
        Command::ReadControllerIndexList,
        Controller::none(),
        None,
        event_tx,
    )
    .await?;

    let mut param = param.unwrap();
    let count = param.get_u16_le() as usize;
    let mut controllers = vec![Controller::none(); count];
    for i in 0..count {
        controllers[i] = Controller(param.get_u16_le());
    }

    Ok(controllers)
}

/// This command is used to retrieve the current state and basic
///	information of a controller. It is typically used right after
///	getting the response to the Read Controller Index List command
///	or an Index Added event.
///
///	The `address` parameter describes the controllers public address
///	and it can be expected that it is set. However in case of single
///	mode Low Energy only controllers it can be `00:00:00:00:00:00`. To
///	power on the controller in this case, it is required to configure
///	a static address using Set Static `address` command first.
///
///	If the public address is set, then it will be used as identity
///	address for the controller. If no public address is available,
///	then the configured static address will be used as identity
///	address.
///
///	In the case of a dual-mode controller with public address that
///	is configured as Low Energy only device (BR/EDR switched off),
///	the static address is used when set and public address otherwise.
///
///	If no short name is set the Short_Name parameter will be all zeroes.
pub async fn get_controller_info(
    socket: &mut ManagementStream,
    controller: Controller,
    event_tx: Option<mpsc::Sender<Response>>,
) -> Result<ControllerInfo> {
    let (_, param) = exec_command(
        socket,
        Command::ReadControllerInfo,
        controller,
        None,
        event_tx,
    )
    .await?;

    let mut param = param.ok_or(Error::NoData)?;

    Ok(ControllerInfo {
        address: param.get_address(),
        bluetooth_version: param.get_u8(),
        manufacturer: param.get_u16_le(),
        supported_settings: param.get_flags_u32_le(),
        current_settings: param.get_flags_u32_le(),
        class_of_device: device_class_from_bytes(param.split_to(3)),
        name: param.split_to(249).get_c_string(),
        short_name: param.get_c_string(),
    })
}

///	This command is used to retrieve a list of currently connected
///	devices.
///
///	For devices using resolvable random addresses with a known
///	identity resolving key, the `address` and `address_type` will
///	contain the identity information.
///
///	This command can only be used when the controller is powered.
pub async fn get_connections(
    socket: &mut ManagementStream,
    controller: Controller,
    event_tx: Option<mpsc::Sender<Response>>,
) -> Result<Vec<(Address, AddressType)>> {
    let (_, param) =
        exec_command(socket, Command::GetConnections, controller, None, event_tx).await?;

    let mut param = param.ok_or(Error::NoData)?;
    let count = param.get_u16_le() as usize;
    let mut connections = Vec::with_capacity(count);

    for _ in 0..count {
        connections.push((param.get_address(), param.get_primitive_u8()));
    }

    Ok(connections)
}

/// This command is used to get connection information.
pub async fn get_connection_info(
    socket: &mut ManagementStream,
    controller: Controller,
    address: Address,
    address_type: AddressType,
    event_tx: Option<mpsc::Sender<Response>>,
) -> Result<ConnectionInfo> {
    let mut param = BytesMut::with_capacity(7);
    param.put_slice(address.as_ref());
    param.put_u8(address_type as u8);

    let (_, param) = exec_command(
        socket,
        Command::GetConnectionInfo,
        controller,
        Some(param.freeze()),
        event_tx,
    )
    .await?;

    let mut param = param.ok_or(Error::NoData)?;
    Ok(ConnectionInfo {
        address: param.get_address(),
        address_type: param.get_primitive_u8(),
        rssi: if param[0] != 127 {
            Some(param.get_i8())
        } else {
            None
        },
        tx_power: if param[0] != 127 {
            Some(param.get_i8())
        } else {
            None
        },
        max_tx_power: if param[0] != 127 {
            Some(param.get_i8())
        } else {
            None
        },
    })
}

/// This command is used to get local and piconet clock information.
pub async fn get_clock_info(
    socket: &mut ManagementStream,
    controller: Controller,
    address: Address,
    address_type: AddressType,
    event_tx: Option<mpsc::Sender<Response>>,
) -> Result<ClockInfo> {
    let mut param = BytesMut::with_capacity(7);
    param.put_slice(address.as_ref());
    param.put_u8(address_type as u8);

    let (_, param) = exec_command(
        socket,
        Command::GetClockInfo,
        controller,
        Some(param.freeze()),
        event_tx,
    )
    .await?;

    let mut param = param.ok_or(Error::NoData)?;

    let address = param.get_address();
    let address_type = param.get_primitive_u8();
    let local_clock = param.get_u32_le();

    let mut piconet_clock = None;
    let mut accuracy = None;

    if address != Address::zero() {
        piconet_clock = Some(param.get_u32_le());
        let accuracy_tmp = param.get_u16_le();
        if accuracy_tmp != 0xFFFF {
            accuracy = Some(accuracy_tmp);
        }
    }

    Ok(ClockInfo {
        address,
        address_type,
        local_clock,
        piconet_clock,
        accuracy,
    })
}

///	This command returns the list of currently unconfigured controllers.
///	Unconfigured controllers added after calling this command can be
///	monitored using the Unconfigured Index Added event.
///
///	An unconfigured controller can either move to a configured state
///	by indicating Unconfigured Index Removed event followed by an
///	Index Added event; or it can be removed from the system which
///	would be indicated by the Unconfigured Index Removed event.
///
///	Only controllers that require configuration will be listed with
///	this command. A controller that is fully configured will not
///	be listed even if it supports configuration changes.
pub async fn get_unconfigured_controller_list(
    socket: &mut ManagementStream,
    event_tx: Option<mpsc::Sender<Response>>,
) -> Result<Vec<Controller>> {
    let (_, param) = exec_command(
        socket,
        Command::ReadUnconfiguredControllerIndexList,
        Controller::none(),
        None,
        event_tx,
    )
    .await?;

    let mut param = param.ok_or(Error::NoData)?;
    let count = param.get_u16_le() as usize;
    let mut controllers = vec![Controller::none(); count];
    for i in 0..count {
        controllers[i] = Controller(param.get_u16_le());
    }

    Ok(controllers)
}

///	This command is used to retrieve the supported configuration
///	options of a controller and the missing configuration options.
///
///	The missing options are required to be configured before the
///	controller is considered fully configured and ready for standard
///	operation. The command is typically used right after getting the
///	response to Read Unconfigured Controller Index List command or
///	Unconfigured Index Added event.
///
///	Supported_Options and Missing_Options is a bitmask with currently
///	the following available bits:
///
///	0	External configuration
///	1	Bluetooth public address configuration
///
///	It is valid to call this command on controllers that do not
///	require any configuration. It is possible that a fully configured
///	controller offers additional support for configuration.
///
///	For example a controller may contain a valid Bluetooth public
///	device address, but also allows to configure it from the host
///	stack. In this case the general support for configurations will
///	be indicated by the Controller Configuration settings. For
///	controllers where no configuration options are available that
///	setting option will not be present.
///
///	When all configurations have been completed and as a result the
///	Missing_Options mask would become empty, then the now ready
///	controller will be announced via Index Added event.
pub async fn get_controller_config_info(
    socket: &mut ManagementStream,
    controller: Controller,
    event_tx: Option<mpsc::Sender<Response>>,
) -> Result<ControllerConfigInfo> {
    let (_, param) = exec_command(
        socket,
        Command::ReadControllerConfigInfo,
        controller,
        None,
        event_tx,
    )
    .await?;

    let mut param = param.ok_or(Error::NoData)?;
    Ok(ControllerConfigInfo {
        manufacturer: param.get_u16_le(),
        supported_options: param.get_flags_u32_le(),
        missing_options: param.get_flags_u32_le(),
    })
}

/// This command returns the list of currently known controllers. It
///	includes configured, unconfigured and alternate controllers.
///
///	Controllers added or removed after calling this command can be
///	be monitored using the Extended Index Added and Extended Index
///	Removed events.
///
///	The existing Index Added, Index Removed, Unconfigured Index Added
///	and Unconfigured Index Removed are no longer sent after this command
///	has been used at least once.
///
///	Instead of calling Read Controller Index List and Read Unconfigured
///	Controller Index List, this command combines all the information
///	and can be used to retrieve the controller list.
///
/// Controllers marked as RAW only operation are currently not listed
///	by this command.
pub async fn get_ext_controller_list(
    socket: &mut ManagementStream,
    event_tx: Option<mpsc::Sender<Response>>,
) -> Result<Vec<(Controller, ControllerType, ControllerBus)>> {
    let (_, param) = exec_command(
        socket,
        Command::ReadExtendedControllerIndexList,
        Controller::none(),
        None,
        event_tx,
    )
    .await?;

    let mut param = param.ok_or(Error::NoData)?;
    let count = param.get_u16_le() as usize;
    let mut index = Vec::with_capacity(count);
    for _ in 0..count {
        index.push((
            Controller(param.get_u16_le()),
            param.get_primitive_u8(),
            param.get_primitive_u8(),
        ));
    }
    Ok(index)
}

/// This command is used to retrieve the current state and basic
///	information of a controller. It is typically used right after
///	getting the response to the Read Controller Index List command
///	or an Index Added event (or its extended counterparts).
///
///	The Address parameter describes the controllers public address
///	and it can be expected that it is set. However in case of single
///	mode Low Energy only controllers it can be 00:00:00:00:00:00. To
///	power on the controller in this case, it is required to configure
///	a static address using Set Static Address command first.
///
///	If the public address is set, then it will be used as identity
///	address for the controller. If no public address is available,
///	then the configured static address will be used as identity
///	address.
///
///	In the case of a dual-mode controller with public address that
///	is configured as Low Energy only device (BR/EDR switched off),
///	the static address is used when set and public address otherwise.
pub async fn get_ext_controller_info(
    socket: &mut ManagementStream,
    controller: Controller,
    event_tx: Option<mpsc::Sender<Response>>,
) -> Result<ControllerInfoExt> {
    let (_, param) = exec_command(
        socket,
        Command::ReadExtendedControllerInfo,
        controller,
        None,
        event_tx,
    )
    .await?;

    let mut param = param.ok_or(Error::NoData)?;

    Ok(ControllerInfoExt {
        address: param.get_address(),
        bluetooth_version: param.get_u8(),
        manufacturer: param.get_u16_le(),
        supported_settings: param.get_flags_u32_le(),
        current_settings: param.get_flags_u32_le(),
        eir_data: {
            let len = param.get_u16_le();
            param.split_to(len as usize)
        },
    })
}

/// If BR/EDR is supported, then BR 1M 1-Slot is supported by
///	default and can also not be deselected. If LE is supported,
///	then LE 1M TX and LE 1M RX are supported by default.
///
///	Disabling BR/EDR completely or respectively LE has no impact
///	on the PHY configuration. It is remembered over power cycles.
pub async fn get_phy_config(
    socket: &mut ManagementStream,
    controller: Controller,
    event_tx: Option<mpsc::Sender<Response>>,
) -> Result<PhyConfig> {
    let (_, param) =
        exec_command(socket, Command::GetPhyConfig, controller, None, event_tx).await?;

    let mut param = param.ok_or(Error::NoData)?;
    Ok(PhyConfig {
        supported_phys: param.get_flags_u32_le(),
        configurable_phys: param.get_flags_u32_le(),
        selected_phys: param.get_flags_u32_le(),
    })
}

/// Currently no Parameter_Type values are defined and an empty list
/// will be returned.
///
/// This command can be used at any time and will return a list of
/// supported default parameters as well as their current value.
pub async fn get_default_runtime_config(
    socket: &mut ManagementStream,
    controller: Controller,
    event_tx: Option<mpsc::Sender<Response>>,
) -> Result<HashMap<RuntimeConfigParameterType, Vec<u8>>> {
    let (_, param) = exec_command(
        socket,
        Command::ReadDefaultRuntimeConfig,
        controller,
        None,
        event_tx,
    )
    .await?;

    let mut param = param.ok_or(Error::NoData)?;
    Ok(param.get_tlv_map())
}

/// This command can be used at any time and will return a list of
/// supported default parameters as well as their current value.
pub async fn get_default_system_config(
    socket: &mut ManagementStream,
    controller: Controller,
    event_tx: Option<mpsc::Sender<Response>>,
) -> Result<HashMap<SystemConfigParameterType, Vec<u8>>> {
    let (_, param) = exec_command(
        socket,
        Command::ReadDefaultSystemConfig,
        controller,
        None,
        event_tx,
    )
    .await?;

    let mut param = param.ok_or(Error::NoData)?;
    Ok(param.get_tlv_map())
}
