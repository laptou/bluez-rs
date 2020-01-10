use std::convert::TryInto;
use std::ffi::CString;

use bytes::*;
use num_traits::FromPrimitive;

pub use params::*;
pub use settings::*;

use crate::{Error, Result};
use crate::Address;
use crate::interface::*;
use crate::interface::class::{DeviceClass, ServiceClasses};
use crate::interface::controller::{Controller, ControllerInfo, ControllerSettings};
use crate::interface::event::Event;
use crate::socket::ManagementSocket;

mod advertising;
mod class;
mod discovery;
mod interact;
mod load;
mod oob;
mod params;
mod query;
mod settings;

type BlueZClientHandler = dyn FnMut(Controller, &Event) -> ();

pub struct BlueZClient {
    socket: ManagementSocket,
    handler: Option<Box<BlueZClientHandler>>,
}

impl BlueZClient {
    pub fn new() -> Result<Self> {
        Ok(BlueZClient {
            socket: ManagementSocket::open()?,
            handler: None,
        })
    }

    pub fn new_with_handler(handler: Box<BlueZClientHandler>) -> Result<Self> {
        Ok(BlueZClient {
            socket: ManagementSocket::open()?,
            handler: Some(handler),
        })
    }

    /// Sets a handler that will be called every time this client processes
    /// an event. CommandComplete and CommandStatus events will NOT reach this handler;
    /// instead their contents can be accessed as the return value of the method
    /// that you called.
    pub fn set_handler(&mut self, handler: Option<Box<BlueZClientHandler>>) {
        self.handler = handler;
    }

    /// Tells the client to check if any new data has been sent in by the kernel.
    /// If you do not call this method, you will not recieve any events except
    /// when you happen to issue a command. If `block` is true, this method
    /// will block until there is a response to read. If `block` is false,
    /// this method will either return a pending response or return `Err(ManagementError::NoData)`,
    /// which in most cases can be safely ignored.
    pub async fn process(&mut self) -> Result<Response> {
        let response = self.socket.receive().await?;

        match &response.event {
            Event::CommandStatus { .. } | Event::CommandComplete { .. } => (),
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
        opcode: Command,
        controller: Controller,
        param: Option<Bytes>,
        callback: F,
    ) -> Result<T> {
        let param = param.unwrap_or(Bytes::new());

        // send request
        self.socket
            .send(Request {
                opcode,
                controller,
                param,
            })
            .await?;

        // loop until we receive a relevant response
        // which is either command complete or command status
        // with the same opcode as the command that we sent
        loop {
            let response = self.process().await?;

            match response.event {
                Event::CommandComplete {
                    status,
                    param,
                    opcode: evt_opcode,
                } if opcode == evt_opcode => {
                    return match status {
                        CommandStatus::Success => callback(response.controller, Some(param)),
                        _ => Err(Error::CommandError { opcode, status }),
                    }
                }
                Event::CommandStatus {
                    status,
                    opcode: evt_opcode,
                } if opcode == evt_opcode => {
                    return match status {
                        CommandStatus::Success => callback(response.controller, None),
                        _ => Err(Error::CommandError { opcode, status }),
                    }
                }
                _ => (),
            }
        }
    }
}
