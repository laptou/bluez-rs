use std::ffi::CString;

use bytes::*;

pub use advertising::*;
pub use load::*;
pub use oob::*;
pub use params::*;
pub use settings::*;

use crate::management::interface::*;
use crate::management::socket::ManagementSocket;
use crate::management::{Error, Result};
use crate::Address;

mod advertising;
mod class;
mod discovery;
mod interact;
mod load;
mod oob;
mod params;
mod query;
mod settings;

pub struct ManagementClient<'a> {
    socket: ManagementSocket,
    handler: Option<ManagementEventHandler<'a>>,
}

pub type ManagementEventHandler<'a> = Box<dyn (FnMut(Controller, &Event)) + Send + 'a>;

impl<'a> ManagementClient<'a> {
    pub fn new() -> Result<Self> {
        Ok(ManagementClient {
            socket: ManagementSocket::open()?,
            handler: None,
        })
    }

    pub fn new_with_handler(handler: ManagementEventHandler<'a>) -> Result<Self> {
        Ok(ManagementClient {
            socket: ManagementSocket::open()?,
            handler: Some(handler),
        })
    }

    /// Sets a handler that will be called every time this client processes
    /// an event. CommandComplete and CommandStatus events will NOT reach this handler;
    /// instead their contents can be accessed as the return value of the method
    /// that you called.
    pub fn set_handler(&mut self, handler: Option<ManagementEventHandler<'a>>) {
        self.handler = handler;
    }

    /// Tells the client to check if any new data has been sent in by the kernel.
    /// If you do not call this method, you will not recieve any events except
    /// when you happen to issue a command.
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
        let param = param.unwrap_or_default();

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
