use crate::mgmt::interface::command::ManagementCommand;
use crate::util::*;

pub struct ManagementRequest {
    pub opcode: ManagementCommand,
    pub controller: u16,
    pub param: Box<Vec<u8>>,
}

impl ManagementRequest {
    pub unsafe fn get_buf(&self) -> Vec<u8> {
        let mut buf: Vec<u8> = Vec::new();

        buf.resize(6 + self.param.len(), 0);

        let buf_slice = buf.as_mut_slice();

        buf.splice(0..2, (self.opcode as u16).to_le_bytes().iter().cloned());
        buf.splice(2..4, self.controller.to_le_bytes().iter().cloned());
        buf.splice(
            4..6,
            (self.param.len() as u16).to_le_bytes().iter().cloned(),
        );
        buf.splice(6.., { &self }.param.iter().cloned());

        buf
    }
}
