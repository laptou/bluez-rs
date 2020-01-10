use enumflags2::BitFlags;

use crate::mgmt::interface::class::from_bytes as class_from_bytes;
use crate::util::bytes_to_c_str;

use super::*;

impl ManagementClient {
    /// This command returns the Management version and revision.
    ///	Besides, being informational the information can be used to
    ///	determine whether certain behavior has changed or bugs fixed
    ///	when interacting with the kernel.
    pub async fn get_mgmt_version(&mut self) -> Result<ManagementVersion> {
        self.exec_command(
            ManagementCommand::ReadVersionInfo,
            Controller::none(),
            None,
            |_, param| {
                let mut param = param.unwrap();
                Ok(ManagementVersion {
                    version: param.get_u8(),
                    revision: param.get_u16_le(),
                })
            },
        )
        .await
    }

    /// This command returns the list of currently known controllers.
    ///	Controllers added or removed after calling this command can be
    ///	monitored using the Index Added and Index Removed events.
    pub async fn get_controller_list(&mut self) -> Result<Vec<Controller>> {
        self.exec_command(
            ManagementCommand::ReadControllerIndexList,
            Controller::none(),
            None,
            |_, param| {
                let mut param = param.unwrap();
                let count = param.get_u16_le() as usize;
                let mut controllers = vec![Controller::none(); count];
                for i in 0..count {
                    controllers[i] = Controller(param.get_u16_le());
                }

                Ok(controllers)
            },
        )
        .await
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
    pub async fn get_controller_info(&mut self, controller: Controller) -> Result<ControllerInfo> {
        self.exec_command(
            ManagementCommand::ReadControllerInfo,
            controller,
            None,
            |_, param| {
                let mut param = param.unwrap();

                Ok(ControllerInfo {
                    address: Address::from_slice(param.split_to(6).as_ref()),
                    bluetooth_version: param.get_u8(),
                    manufacturer: param.split_to(2).as_ref().try_into().unwrap(),
                    supported_settings: ControllerSettings::from_bits_truncate(param.get_u32_le()),
                    current_settings: ControllerSettings::from_bits_truncate(param.get_u32_le()),
                    class_of_device: class_from_bytes(param.split_to(3).to_bytes()),
                    name: bytes_to_c_str(param.split_to(249)),
                    short_name: bytes_to_c_str(param),
                })
            },
        )
        .await
    }

    /// This command is used to get connection information.
    pub async fn get_connection_info(
        &mut self,
        controller: Controller,
        address: Address,
        address_type: AddressType,
    ) -> Result<ConnectionInfo> {
        let mut param = BytesMut::with_capacity(7);
        param.put_slice(address.as_ref());
        param.put_u8(address_type as u8);

        self.exec_command(
            ManagementCommand::GetConnectionInfo,
            controller,
            Some(param.to_bytes()),
            |_, param| {
                let mut param = param.unwrap();
                Ok(ConnectionInfo {
                    address: Address::from_slice(param.split_to(6).as_ref()),
                    address_type: FromPrimitive::from_u8(param.get_u8()).unwrap(),
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
            },
        )
        .await
    }

    /// This command is used to get local and piconet clock information.
    pub async fn get_clock_info(
        &mut self,
        controller: Controller,
        address: Address,
        address_type: AddressType,
    ) -> Result<ClockInfo> {
        let mut param = BytesMut::with_capacity(7);
        param.put_slice(address.as_ref());
        param.put_u8(address_type as u8);

        self.exec_command(
            ManagementCommand::GetClockInfo,
            controller,
            Some(param.to_bytes()),
            |_, param| {
                let mut param = param.unwrap();

                let address = Address::from_slice(param.split_to(6).as_ref());
                let address_type = FromPrimitive::from_u8(param.get_u8()).unwrap();
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
            },
        )
        .await
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
    pub async fn get_unconfigured_controller_list(&mut self) -> Result<Vec<Controller>> {
        self.exec_command(
            ManagementCommand::ReadUnconfiguredControllerIndexList,
            Controller::none(),
            None,
            |_, param| {
                let mut param = param.unwrap();
                let count = param.get_u16_le() as usize;
                let mut controllers = vec![Controller::none(); count];
                for i in 0..count {
                    controllers[i] = Controller(param.get_u16_le());
                }

                Ok(controllers)
            },
        )
        .await
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
    ///		0	External configuration
    ///		1	Bluetooth public address configuration
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
        &mut self,
        controller: Controller,
    ) -> Result<ControllerConfigInfo> {
        self.exec_command(
            ManagementCommand::ReadControllerConfigInfo,
            controller,
            None,
            |_, param| {
                let mut param = param.unwrap();
                Ok(ControllerConfigInfo {
                    manufacturer: param.split_to(2).as_ref().try_into().unwrap(),
                    supported_options: BitFlags::from_bits_truncate(param.get_u32_le()),
                    missing_options: BitFlags::from_bits_truncate(param.get_u32_le()),
                })
            },
        )
        .await
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
        &mut self,
    ) -> Result<Vec<(Controller, ControllerType, ControllerBus)>> {
        self.exec_command(
            ManagementCommand::ReadExtendedControllerIndexList,
            Controller::none(),
            None,
            |_, param| {
                let mut param = param.unwrap();
                let count = param.get_u16_le() as usize;
                let mut index = Vec::with_capacity(count);
                for _ in 0..count {
                    index.push((
                        Controller(param.get_u16_le()),
                        FromPrimitive::from_u8(param.get_u8()).unwrap(),
                        FromPrimitive::from_u8(param.get_u8()).unwrap(),
                    ));
                }
                Ok(index)
            },
        )
        .await
    }
}
