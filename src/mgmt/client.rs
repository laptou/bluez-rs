use bytes::{Buf, Bytes, IntoBuf};

use crate::mgmt::interface::{ManagementCommand, ManagementCommandStatus, ManagementRequest, ManagementResponse, Version};
use crate::mgmt::interface::controller::Controller;
use crate::mgmt::interface::event::ManagementEvent;
use crate::mgmt::ManagementError;
use crate::mgmt::socket::ManagementSocket;

pub struct ManagementClient {
    socket: ManagementSocket,
    handler: Option<Box<dyn ManagementHandler>>,
}

pub trait ManagementHandler {}

impl ManagementClient {
    pub fn new() -> Self {
        // todo: fix that unwrap()
        ManagementClient {
            socket: ManagementSocket::open().unwrap(),
            handler: None,
        }
    }

    pub fn new_with_handler(handler: Box<dyn ManagementHandler>) -> Self {
        // todo: fix that unwrap()
        ManagementClient {
            socket: ManagementSocket::open().unwrap(),
            handler: Some(handler),
        }
    }

    pub async fn get_mgmt_version(&mut self) -> Result<Version, ManagementError> {
        self.socket.send(ManagementRequest {
            opcode: ManagementCommand::ReadVersionInfo,
            controller: Controller::none(),
            param: Bytes::default(),
        }).await?;

        let response = self.socket.receive().await?;

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