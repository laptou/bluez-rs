use std::convert::TryInto;
use std::ffi::CString;

use bytes::*;
use num_traits::FromPrimitive;

pub use params::*;
pub use settings::*;

use crate::Address;
use crate::mgmt::{ManagementError, Result};
use crate::mgmt::interface::*;
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

pub struct ManagementClient {
    socket: ManagementSocket,
    handler: Option<Box<dyn FnMut(Controller, &ManagementEvent) -> ()>>,
}

impl ManagementClient {
    pub fn new() -> Result<Self> {
        Ok(ManagementClient {
            socket: ManagementSocket::open()?,
            handler: None,
        })
    }

    pub fn new_with_handler(
        handler: Box<dyn FnMut(Controller, &ManagementEvent) -> ()>,
    ) -> Result<Self> {
        Ok(ManagementClient {
            socket: ManagementSocket::open()?,
            handler: Some(handler),
        })
    }

    /// Sets a handler that will be called every time this client processes
    /// an event. CommandComplete and CommandStatus events will NOT reach this handler;
    /// instead their contents can be accessed as the return value of the method
    /// that you called.
    pub fn set_handler(
        &mut self,
        handler: Option<Box<dyn FnMut(Controller, &ManagementEvent) -> ()>>,
    ) {
        self.handler = handler;
    }

    /// Tells the client to check if any new data has been sent in by the kernel.
    /// If you do not call this method, you will not recieve any events except
    /// when you happen to issue a command. If `block` is true, this method
    /// will block until there is a response to read. If `block` is false,
    /// this method will either return a pending response or return `Err(ManagementError::NoData)`,
    /// which in most cases can be safely ignored.
    pub async fn process(&mut self, block: bool) -> Result<ManagementResponse> {
        let response = self.socket.receive(block).await?;

        match &response.event {
            ManagementEvent::CommandStatus { .. } | ManagementEvent::CommandComplete { .. } => (),
            _ => {
                if let Some(handler) = &mut self.handler {
                    (handler)(response.controller, &response.event)
                }
            }
        }

        Ok(response)
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
            let response = self.process(true).await?;

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
}
