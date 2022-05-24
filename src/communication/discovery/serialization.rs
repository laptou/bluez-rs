use crate::communication::{Uuid128, Uuid16, Uuid32};
use crate::util::BufExt;

use bytes::{Buf, BufMut, Bytes, BytesMut};
use num_traits::FromPrimitive;
use std::ffi::OsString;
use std::os::unix::ffi::{OsStrExt, OsStringExt};

pub trait ToBuf {
    fn to_buf<B: BufMut>(&self, buf: &mut B);
}

#[derive(Debug)]
pub(super) struct Pdu {
    pub(super) id: PduId,
    pub(super) txn: u16,
    pub(super) parameter: Bytes,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromPrimitive, ToPrimitive)]
pub enum PduId {
    ErrorResponse = 0x01,
    ServiceSearchRequest,
    ServiceSearchResponse,
    ServiceAttributeRequest,
    ServiceAttributeResponse,
    ServiceSearchAttributeRequest,
    ServiceSearchAttributeResponse,
}

impl Pdu {
    pub fn with_parameter<F: ToBuf>(id: PduId, txn: u16, parameter: F) -> Self {
        let mut buf = BytesMut::new();
        parameter.to_buf(&mut buf);
        Self {
            id,
            txn,
            parameter: buf.freeze(),
        }
    }
}

impl<B: Buf> From<&mut B> for Pdu {
    fn from(buf: &mut B) -> Self {
        Pdu {
            id: FromPrimitive::from_u8(buf.get_u8()).unwrap(),
            txn: buf.get_u16(),
            parameter: {
                let param_size = buf.get_u16() as usize;
                buf.copy_to_bytes(param_size)
            },
        }
    }
}

impl ToBuf for Pdu {
    fn to_buf<B: BufMut>(&self, buf: &mut B) {
        buf.put_u8(self.id as u8);
        buf.put_u16(self.txn);
        let param_size = self.parameter.len() as u16;
        buf.put_u16(param_size);
        buf.put(&self.parameter[..]);
    }
}

#[derive(Debug, Clone)]
pub enum DataElement {
    Nil,
    Uint8(u8),
    Uint16(u16),
    Uint32(u32),
    Uint64(u64),
    Uint128(u128),
    Int8(i8),
    Int16(i16),
    Int32(i32),
    Int64(i64),
    Int128(i128),
    Uuid16(Uuid16),
    Uuid32(Uuid32),
    Uuid128(Uuid128),
    Bool(bool),
    String(OsString),
    Url(OsString),
    // Data element sequence, a data element whose data field is a sequence of
    // data elements
    Sequence(Vec<DataElement>),
    // Data element alternative, data element whose data field is a sequence of
    // data elements from which one data element is to be selected.
    Alternative(Vec<DataElement>),
}

impl<B: Buf> From<&mut B> for DataElement {
    fn from(buf: &mut B) -> Self {
        let desc = buf.get_u8();
        let type_desc = (desc & 0b11111000) >> 3;
        let size_desc = desc & 0b00000111;

        match type_desc {
            0 => Self::Nil,
            1 => match size_desc {
                0 => Self::Uint8(buf.get_u8()),
                1 => Self::Uint16(buf.get_u16()),
                2 => Self::Uint32(buf.get_u32()),
                3 => Self::Uint64(buf.get_u64()),
                4 => Self::Uint128(buf.get_u128()),
                _ => panic!("invalid size descriptor"),
            },
            2 => match size_desc {
                0 => Self::Int8(buf.get_i8()),
                1 => Self::Int16(buf.get_i16()),
                2 => Self::Int32(buf.get_i32()),
                3 => Self::Int64(buf.get_i64()),
                4 => Self::Int128(buf.get_i128()),
                _ => panic!("invalid size descriptor"),
            },
            3 => match size_desc {
                1 => Self::Uuid16(Uuid16(buf.get_u16())),
                2 => Self::Uuid32(Uuid32(buf.get_u32())),
                4 => Self::Uuid128(Uuid128(buf.get_u128())),
                _ => panic!("invalid size descriptor"),
            },
            4 => {
                let size = match size_desc {
                    5 => buf.get_u8() as usize,
                    6 => buf.get_u16() as usize,
                    7 => buf.get_u32() as usize,
                    _ => panic!("invalid size descriptor"),
                };
                let bytes = buf.get_vec_u8(size);
                Self::String(OsString::from_vec(bytes))
            }
            5 => match size_desc {
                0 => Self::Bool(buf.get_bool()),
                _ => panic!("invalid size descriptor"),
            },
            6 => {
                let size = match size_desc {
                    5 => buf.get_u8() as usize,
                    6 => buf.get_u16() as usize,
                    7 => buf.get_u32() as usize,
                    _ => panic!("invalid size descriptor"),
                };

                let mut seq_buf = buf.copy_to_bytes(size);
                let mut seq = vec![];

                while seq_buf.len() > 0 {
                    seq.push(DataElement::from(&mut seq_buf))
                }

                Self::Sequence(seq)
            }
            7 => {
                let size = match size_desc {
                    5 => buf.get_u8() as usize,
                    6 => buf.get_u16() as usize,
                    7 => buf.get_u32() as usize,
                    _ => panic!("invalid size descriptor"),
                };

                let mut seq_buf = buf.copy_to_bytes(size);
                let mut seq = vec![];

                while seq_buf.len() > 0 {
                    seq.push(DataElement::from(&mut seq_buf))
                }

                Self::Alternative(seq)
            }
            8 => {
                let size = match size_desc {
                    5 => buf.get_u8() as usize,
                    6 => buf.get_u16() as usize,
                    7 => buf.get_u32() as usize,
                    _ => panic!("invalid size descriptor"),
                };
                let bytes = buf.get_vec_u8(size);
                Self::Url(OsString::from_vec(bytes))
            }
            _ => panic!("invalid size descriptor"),
        }
    }
}

impl DataElement {
    pub(super) fn to_buf<B: BufMut>(&self, buf: &mut B) {
        let (type_desc, size_desc, size): (u8, Option<u8>, usize) = match self {
            DataElement::Nil => (0, Some(0), 0),
            DataElement::Uint8(_) => (1, Some(0), 0),
            DataElement::Uint16(_) => (1, Some(1), 0),
            DataElement::Uint32(_) => (1, Some(2), 0),
            DataElement::Uint64(_) => (1, Some(3), 0),
            DataElement::Uint128(_) => (1, Some(4), 0),
            DataElement::Int8(_) => (2, Some(0), 0),
            DataElement::Int16(_) => (2, Some(1), 0),
            DataElement::Int32(_) => (2, Some(2), 0),
            DataElement::Int64(_) => (2, Some(3), 0),
            DataElement::Int128(_) => (2, Some(4), 0),
            DataElement::Uuid16(_) => (3, Some(1), 0),
            DataElement::Uuid32(_) => (3, Some(2), 0),
            DataElement::Uuid128(_) => (3, Some(4), 0),
            DataElement::String(s) => (4, None, s.len()),
            DataElement::Bool(_) => (5, None, 0),
            DataElement::Sequence(s) => (
                6,
                None,
                s.iter()
                    .map(|i| {
                        let mut b = BytesMut::new();
                        i.to_buf(&mut b);
                        b.len()
                    })
                    .sum(),
            ),
            DataElement::Alternative(s) => (
                7,
                None,
                s.iter()
                    .map(|i| {
                        let mut b = BytesMut::new();
                        i.to_buf(&mut b);
                        b.len()
                    })
                    .sum(),
            ),
            DataElement::Url(s) => (8, None, s.len()),
        };

        let size_desc = match size_desc {
            Some(size_desc) => size_desc,
            None => {
                if size < u8::MAX as usize {
                    5
                } else if size < u16::MAX as usize {
                    6
                } else if size < u32::MAX as usize {
                    7
                } else {
                    panic!("size of data too large");
                }
            }
        };

        let header = (type_desc << 3) | size_desc;

        buf.put_u8(header);

        match size_desc {
            5 => buf.put_u8(size as u8),
            6 => buf.put_u16(size as u16),
            7 => buf.put_u32(size as u32),
            _ => {}
        };

        match self {
            DataElement::Nil => {}
            DataElement::Uint8(v) => buf.put_u8(*v),
            DataElement::Uint16(v) => buf.put_u16(*v),
            DataElement::Uint32(v) => buf.put_u32(*v),
            DataElement::Uint64(v) => buf.put_u64(*v),
            DataElement::Uint128(v) => buf.put_u128(*v),
            DataElement::Int8(v) => buf.put_i8(*v),
            DataElement::Int16(v) => buf.put_i16(*v),
            DataElement::Int32(v) => buf.put_i32(*v),
            DataElement::Int64(v) => buf.put_i64(*v),
            DataElement::Int128(v) => buf.put_i128(*v),
            DataElement::Uuid16(v) => buf.put_u16(v.0),
            DataElement::Uuid32(v) => buf.put_u32(v.0),
            DataElement::Uuid128(v) => buf.put_u128(v.0),
            DataElement::Bool(v) => buf.put_u8(*v as u8),
            DataElement::String(v) | DataElement::Url(v) => buf.put_slice(v.as_bytes()),
            DataElement::Sequence(v) | DataElement::Alternative(v) => {
                for vi in v {
                    vi.to_buf(buf);
                }
            }
        };
    }
}
