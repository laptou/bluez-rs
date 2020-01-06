use bytes::{Buf, Bytes, IntoBuf};

use crate::mgmt::interface::{ManagementCommand, ManagementCommandStatus, ManagementRequest, ManagementResponse, Version};
use crate::mgmt::interface::controller::Controller;
use crate::mgmt::interface::event::ManagementEvent;
use crate::mgmt::ManagementError;
use crate::mgmt::socket::ManagementSocket;

pub struct BlueZClient {
    socket: ManagementSocket,
    events: Vec<ManagementResponse>,
}

impl BlueZClient {
    pub fn new() -> Self {
        // todo: fix that unwrap()
        BlueZClient {
            socket: ManagementSocket::open().unwrap(),
            events: vec![],
        }
    }

    pub async fn event_loop(&mut self) {
        loop {
            let result = self.socket.receive().await.unwrap();
            self.events.push(result);
        }
    }

    pub async fn version(&mut self) -> Result<Version, ManagementError> {
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