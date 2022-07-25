use bytes::*;

use crate::management::interface::command::Command;
use crate::management::interface::controller::Controller;

/// A command that is ready to be sent to the management API.
#[derive(Debug)]
pub struct Request {
    pub opcode: Command,
    pub controller: Controller,
    pub param: Bytes,
}

impl From<Request> for Bytes {
    fn from(val: Request) -> Self {
        let mut buf = BytesMut::with_capacity(6 + val.param.len());

        buf.put_u16_le(val.opcode as u16);
        buf.put_u16_le(val.controller.into());
        buf.put_u16_le(val.param.len() as u16);
        buf.put(val.param);

        buf.freeze()
    }
}
