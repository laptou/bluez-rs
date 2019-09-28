use std::future::Future;

use bitflags::_core::time::Duration;
use bytes::{Buf, Bytes, IntoBuf};

use crate::mgmt::interface::{ManagementCommand, ManagementCommandStatus, ManagementRequest, Version};
use crate::mgmt::interface::controller::Controller;
use crate::mgmt::interface::event::ManagementEvent;
use crate::mgmt::ManagementError;
use crate::mgmt::socket::ManagementSocket;

pub struct BluezClient {
    socket: ManagementSocket,
    timeout: i32,
}

impl BluezClient {
    pub fn new() -> Self {
        // todo: fix that unwrap()
        BluezClient {
            socket: ManagementSocket::open().unwrap(),
            timeout: 5000,
        }
    }

    pub async fn version(&mut self) -> Result<Version, ManagementError> {
        self.socket.send(ManagementRequest {
            opcode: ManagementCommand::ReadVersionInfo,
            controller: Controller::none(),
            param: Bytes::default(),
        }).await?;

        let response = self.socket.receive(self.timeout).await?;

        match response.event {
            ManagementEvent::CommandComplete { status, param, opcode } => {
                match status {
                    ManagementCommandStatus::Success => {
                        let mut cursor = param.into_buf();

                        Ok(Version {
                            version: cursor.get_u8(),
                            revision: cursor.get_u16_le(),
                        })
                    }
                    _ => Err(ManagementError::CommandError {
                        opcode,
                        status,
                    })
                }
            }
            _ => panic!()
        }
    }
}