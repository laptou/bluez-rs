use bytes::*;

use crate::management::interface::command::Command;
use crate::management::interface::controller::Controller;

#[derive(Debug)]
pub struct Request {
    pub opcode: Command,
    pub controller: Controller,
    pub param: Bytes,
}

impl Into<Bytes> for Request {
    fn into(self) -> Bytes {
        let mut buf = BytesMut::with_capacity(6 + self.param.len());

        buf.put_u16_le(self.opcode as u16);
        buf.put_u16_le(self.controller.into());
        buf.put_u16_le(self.param.len() as u16);
        buf.put(self.param);

        buf.freeze()
    }
}
