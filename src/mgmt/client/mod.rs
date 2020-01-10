use std::convert::TryInto;
use std::ffi::CString;

use bytes::*;
use num_traits::FromPrimitive;

pub use params::*;
pub use settings::*;

use crate::Address;
use crate::mgmt::{ManagementError, Result};
use crate::mgmt::interface::{ManagementCommand, ManagementCommandStatus, ManagementRequest};
use crate::mgmt::interface::class::{DeviceClass, ServiceClasses};
use crate::mgmt::interface::controller::{Controller, ControllerInfo, ControllerSettings};
use crate::mgmt::interface::event::ManagementEvent;
use crate::mgmt::socket::ManagementSocket;

mod advertising;
mod class;
mod discovery;
mod interact;
mod load;
mod oob;
mod params;
mod query;
mod settings;

pub struct ManagementClient<H>
where
    H: FnMut(Controller, ManagementEvent) -> (),
{
    socket: ManagementSocket,
    handler: H,
}

impl<H> ManagementClient<H>
where
    H: FnMut(Controller, ManagementEvent) -> (),
{
    pub fn new(handler: H) -> Result<Self> {
        Ok(ManagementClient {
            socket: ManagementSocket::open()?,
            handler,
        })
    }

    #[inline]
    async fn exec_command<F: FnOnce(Controller, Option<Bytes>) -> Result<T>, T>(
        &mut self,
        opcode: ManagementCommand,
        controller: Controller,
        param: Option<Bytes>,
        callback: F,
    ) -> Result<T> {
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
                _ => (self.handler)(response.controller, response.event),
            }
        }
    }
}
