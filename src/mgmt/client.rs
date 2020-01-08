use std::marker::PhantomData;

use bytes::*;

use crate::mgmt::interface::{
    ManagementCommand, ManagementCommandStatus, ManagementRequest, ManagementResponse, Version,
};
use crate::mgmt::interface::controller::Controller;
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
        callback: F) -> Result<T, ManagementError> {
        let param = param.unwrap_or(Bytes::new());

        // send request
        self.socket
            .send(ManagementRequest { opcode, controller, param })
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
                } if opcode == evt_opcode => return match status {
                    ManagementCommandStatus::Success => callback(response.controller, Some(param)),
                    _ => Err(ManagementError::CommandError { opcode, status }),
                },
                ManagementEvent::CommandStatus {
                    status,
                    opcode: evt_opcode,
                } if opcode == evt_opcode => return match status {
                    ManagementCommandStatus::Success => callback(response.controller, None),
                    _ => Err(ManagementError::CommandError { opcode, status }),
                },
                _ => ()
            }
        }
    }

    pub async fn get_mgmt_version(&mut self) -> Result<Version, ManagementError> {
        self.exec_command(ManagementCommand::ReadVersionInfo,
                          Controller::none(),
                          None,
                          |_, param| {
                              let mut param = param.unwrap();
                              Ok(Version {
                                  version: param.get_u8(),
                                  revision: param.get_u16_le(),
                              })
                          }).await
    }
}
