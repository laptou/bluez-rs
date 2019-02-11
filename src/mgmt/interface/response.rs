use num_traits::FromPrimitive;

use crate::mgmt::interface::command::ManagementEvent;
use crate::mgmt::ManagementError;
use crate::util::*;

pub struct ManagementResponse {
    pub event: ManagementEvent,
    pub controller: u16,
}

impl ManagementResponse {
    pub unsafe fn from_buf(buf: &Vec<u8>) -> Result<ManagementResponse, failure::Error> {
        let evt_code = read_u16_le(buf, 0);
        let controller = read_u16_le(buf, 2);
        let param_size = read_u16_le(buf, 4) as usize;

        const HEADER: usize = 6; // header size is 6 bytes

        return Ok(ManagementResponse {
            controller,
            event: match evt_code {
                0x0001 => ManagementEvent::CommandComplete {
                    opcode: FromPrimitive::from_u16(read_u16_le(buf, HEADER)).ok_or(
                        ManagementError::UnknownCommand {
                            cmd: read_u16_le(buf, HEADER),
                        },
                    )?,
                    status: FromPrimitive::from_u8(buf[HEADER + 2]).ok_or(
                        ManagementError::UnknownStatus {
                            status: buf[HEADER + 2],
                        },
                    )?,
                    param: Box::new(buf[HEADER + 3..HEADER + param_size].to_vec()),
                },
                0x0002 => ManagementEvent::CommandStatus {
                    opcode: FromPrimitive::from_u16(read_u16_le(buf, HEADER)).ok_or(
                        ManagementError::UnknownCommand {
                            cmd: read_u16_le(buf, HEADER),
                        },
                    )?,
                    status: FromPrimitive::from_u8(buf[HEADER + 2]).ok_or(
                        ManagementError::UnknownStatus {
                            status: buf[HEADER + 2],
                        },
                    )?,
                },
                0x0003 => ManagementEvent::ControllerError { code: buf[HEADER] },
                0x0004 => ManagementEvent::IndexAdded,
                0x0005 => ManagementEvent::IndexRemoved,
                0x0006 => ManagementEvent::NewSettings {
                    settings: unimplemented!(),
                },
                0x0007 => ManagementEvent::ClassOfDeviceChanged {
                    class: unimplemented!(),
                },
                0x0008 => {
                    let name = read_str(&buf, HEADER, 249);
                    let short_name = read_str(&buf, HEADER + 249, 11);

                    ManagementEvent::LocalNameChanged { name, short_name }
                }
                0x0009 => unimplemented!(),
                0x000A => unimplemented!(),
                0x000B => unimplemented!(),
                0x000C => unimplemented!(),
                0x000D => unimplemented!(),
                0x000E => unimplemented!(),
                0x000F => unimplemented!(),
                0x0010 => unimplemented!(),
                0x0011 => unimplemented!(),
                0x0012 => unimplemented!(),
                0x0013 => unimplemented!(),
                0x0014 => unimplemented!(),
                0x0015 => unimplemented!(),
                0x0016 => unimplemented!(),
                0x0017 => unimplemented!(),
                _ => unreachable!(),
            },
        });
    }
}
