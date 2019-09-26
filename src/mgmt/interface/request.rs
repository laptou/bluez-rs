use bytes::{BufMut, Bytes, BytesMut};

use crate::mgmt::interface::command::ManagementCommand;
use crate::mgmt::interface::controller::Controller;

pub struct ManagementRequest {
    pub opcode: ManagementCommand,
    pub controller: Controller,
    pub param: Bytes,
}

impl Into<Bytes> for ManagementRequest {
    fn into(self) -> Bytes {
        let mut buf = BytesMut::with_capacity(6 + self.param.len());

        buf.put_u16_le(self.opcode as u16);
        buf.put_u16_le(self.controller.into());
        buf.put_u16_le(self.param.len() as u16);
        buf.put(self.param);

        buf.freeze()
    }
}
