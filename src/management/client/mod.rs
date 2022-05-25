use std::ffi::CString;

use bytes::*;

pub use advertising::*;
pub use class::*;
pub use discovery::*;
pub use interact::*;
pub use load::*;
pub use oob::*;
pub use params::*;
pub use query::*;
pub use settings::*;

use tokio::sync::mpsc;

use crate::management::interface::*;
use crate::management::stream::ManagementStream;
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

async fn exec_command(
    socket: &mut ManagementStream,
    opcode: Command,
    controller: Controller,
    param: Option<Bytes>,
    mut event_tx: Option<mpsc::Sender<Response>>,
) -> Result<(Controller, Option<Bytes>)> {
    let param = param.unwrap_or(Bytes::new());

    // send request
    socket
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
        let response = socket.receive().await?;

        match response.event {
            Event::CommandComplete {
                status,
                param,
                opcode: evt_opcode,
            } if opcode == evt_opcode => {
                return match status {
                    CommandStatus::Success => Ok((response.controller, Some(param))),
                    _ => Err(Error::CommandError { opcode, status }),
                }
            }

            Event::CommandStatus {
                status,
                opcode: evt_opcode,
            } if opcode == evt_opcode => {
                return match status {
                    CommandStatus::Success => Ok((response.controller, None)),
                    _ => Err(Error::CommandError { opcode, status }),
                }
            }

            _ => {
                if let Some(event_tx) = &mut event_tx {
                    let _ = event_tx.send(response).await;
                }
            }
        }
    }
}
