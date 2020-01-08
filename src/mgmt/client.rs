use std::convert::TryInto;
use std::ffi::{OsStr, OsString};
use std::marker::PhantomData;
use std::os::unix::ffi::OsStringExt;

use bytes::*;

use crate::Address;
use crate::mgmt::interface::{
    ManagementCommand, ManagementCommandStatus, ManagementRequest, ManagementResponse, ManagementVersion,
};
use crate::mgmt::interface::controller::{Controller, ControllerInfo, ControllerSettings};
use crate::mgmt::interface::event::ManagementEvent;
use crate::mgmt::ManagementError;
use crate::mgmt::socket::ManagementSocket;

pub struct ManagementClient {
    socket: ManagementSocket,
}

impl ManagementClient {
    pub fn new() -> Self {
        // todo: fix that unwrap()
        ManagementClient {
            socket: ManagementSocket::open().unwrap(),
        }
    }

    #[inline]
    async fn exec_command<F: FnOnce(Controller, Option<Bytes>) -> Result<T, ManagementError>, T>(
        &mut self,
        opcode: ManagementCommand,
        controller: Controller,
        param: Option<Bytes>,
        callback: F,
    ) -> Result<T, ManagementError> {
        let param = param.unwrap_or(Bytes::new());

        // send request
        self.socket
            .send(ManagementRequest {
                opcode,
                controller,
                param,
            })
            .await?;

        // loop until we receive a relevant response
        // which is either command complete or command status
        // with the same opcode as the command that we sent
        loop {
            let response = self.socket.receive().await?;

            // if we got an error, just send that back to the user
            // otherwise, give the data received to our callback fn
            match response.event {
                ManagementEvent::CommandComplete {
                    status,
                    param,
                    opcode: evt_opcode,
                } if opcode == evt_opcode => {
                    return match status {
                        ManagementCommandStatus::Success => {
                            callback(response.controller, Some(param))
                        }
                        _ => Err(ManagementError::CommandError { opcode, status }),
                    }
                }
                ManagementEvent::CommandStatus {
                    status,
                    opcode: evt_opcode,
                } if opcode == evt_opcode => {
                    return match status {
                        ManagementCommandStatus::Success => callback(response.controller, None),
                        _ => Err(ManagementError::CommandError { opcode, status }),
                    }
                }
                _ => (),
            }
        }
    }

    /// This command returns the Management version and revision.
    //	Besides, being informational the information can be used to
    //	determine whether certain behavior has changed or bugs fixed
    //	when interacting with the kernel.
    pub async fn get_mgmt_version(&mut self) -> Result<ManagementVersion, ManagementError> {
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
    //	Controllers added or removed after calling this command can be
    //	monitored using the Index Added and Index Removed events.
    pub async fn get_controller_list(&mut self) -> Result<Vec<Controller>, ManagementError> {
        self.exec_command(
            ManagementCommand::ReadControllerIndexList,
            Controller::none(),
            None,
            |_, param| {
                let mut param = param.unwrap();
                let mut count = param.get_u16_le() as usize;
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
    //	information of a controller. It is typically used right after
    //	getting the response to the Read Controller Index List command
    //	or an Index Added event.
    //
    //	The Address parameter describes the controllers public address
    //	and it can be expected that it is set. However in case of single
    //	mode Low Energy only controllers it can be 00:00:00:00:00:00. To
    //	power on the controller in this case, it is required to configure
    //	a static address using Set Static Address command first.
    //
    //	If the public address is set, then it will be used as identity
    //	address for the controller. If no public address is available,
    //	then the configured static address will be used as identity
    //	address.
    //
    //	In the case of a dual-mode controller with public address that
    //	is configured as Low Energy only device (BR/EDR switched off),
    //	the static address is used when set and public address otherwise.
    //
    //	If no short name is set the Short_Name parameter will be empty.
    pub async fn get_controller_info(&mut self, controller: Controller) -> Result<ControllerInfo, ManagementError> {
        self.exec_command(ManagementCommand::ReadControllerInfo, controller, None, |_, param| {
            let mut param = param.unwrap();

            let address = Address::from_slice(param.split_off(6).as_ref());

            let bluetooth_version = param.get_u8();
            let manufacturer: [u8; 2] = param.split_off(2).as_ref().try_into().unwrap();
            let supported_settings = ControllerSettings::from_bits_truncate(param.get_u32_le());
            let current_settings = ControllerSettings::from_bits_truncate(param.get_u32_le());
            let class_of_device: [u8; 3] = param.split_off(3).as_ref().try_into().unwrap();
            let name = OsString::from_vec(param.split_off(249).to_vec());
            let short_name = OsString::from_vec(param.to_vec());

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
        }, )
            .await
    }
}
