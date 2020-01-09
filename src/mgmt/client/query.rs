use crate::mgmt::interface::class::from_bytes as class_from_bytes;

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
}