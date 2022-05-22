use std::{
    ffi::OsString,
    fmt::Debug,
    io::{Read, Write},
    os::unix::prelude::{OsStrExt, OsStringExt},
};

use bytes::{Buf, BufMut, Bytes, BytesMut};
use num_traits::FromPrimitive;

use crate::util::BufExtBlueZ;

use super::stream::BluetoothStream;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Uuid16(u16);

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Uuid32(u32);

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Uuid128(u128);

const BASE_UUID: u128 = 0x00000000_0000_1000_8000_00805F9B34FB;
const BASE_UUID_FACTOR: u128 = 2 ^ 96;

impl Into<Uuid32> for Uuid16 {
    fn into(self) -> Uuid32 {
        Uuid32(self.0 as u32)
    }
}

impl Into<Uuid128> for Uuid16 {
    fn into(self) -> Uuid128 {
        Uuid128((self.0 as u128) * BASE_UUID_FACTOR + BASE_UUID)
    }
}

impl Into<Uuid128> for Uuid32 {
    fn into(self) -> Uuid128 {
        Uuid128((self.0 as u128) * BASE_UUID_FACTOR + BASE_UUID)
    }
}

impl Debug for Uuid16 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:04x}", self.0)
    }
}

impl Debug for Uuid32 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let bytes = u32::to_le_bytes(self.0);
        write!(
            f,
            "{:02x}{:02x}-{:02x}{:02x}",
            bytes[3], bytes[2], bytes[1], bytes[0]
        )
    }
}

impl Debug for Uuid128 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let bytes = u128::to_le_bytes(self.0);
        write!(
            f,
            "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
            bytes[15], bytes[14], bytes[13], bytes[12], bytes[11], bytes[10], bytes[9], bytes[8],
            bytes[7], bytes[6], bytes[5], bytes[4], bytes[3], bytes[2], bytes[1], bytes[0]
        )
    }
}

trait ToBuf {
    fn to_buf<B: BufMut>(&self, buf: &mut B);
}

#[derive(Debug)]
struct Pdu {
    id: PduId,
    txn: u16,
    parameter: Bytes,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromPrimitive, ToPrimitive)]
enum PduId {
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
                let param_size = buf.get_u8() as usize;
                buf.copy_to_bytes(param_size)
            },
        }
    }
}

impl ToBuf for Pdu {
    fn to_buf<B: BufMut>(&self, buf: &mut B) {
        buf.put_u8(self.id as u8);
        buf.put_u16(self.txn);
        buf.put_u16(self.parameter.len() as u16);
        buf.put(&self.parameter[..]);
    }
}

#[derive(Debug, Clone)]
enum DataElement {
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

                Self::Sequence((0..size).map(|_| DataElement::from(&mut *buf)).collect())
            }
            7 => {
                let size = match size_desc {
                    5 => buf.get_u8() as usize,
                    6 => buf.get_u16() as usize,
                    7 => buf.get_u32() as usize,
                    _ => panic!("invalid size descriptor"),
                };

                Self::Alternative((0..size).map(|_| DataElement::from(&mut *buf)).collect())
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
    fn to_buf<B: BufMut>(&self, buf: &mut B) {
        let (type_desc, mut size_desc, size): (u8, u8, usize) = match self {
            DataElement::Nil => (0, 0, 0),
            DataElement::Uint8(_) => (1, 0, 0),
            DataElement::Uint16(_) => (1, 1, 0),
            DataElement::Uint32(_) => (1, 2, 0),
            DataElement::Uint64(_) => (1, 3, 0),
            DataElement::Uint128(_) => (1, 4, 0),
            DataElement::Int8(_) => (2, 0, 0),
            DataElement::Int16(_) => (2, 1, 0),
            DataElement::Int32(_) => (2, 2, 0),
            DataElement::Int64(_) => (2, 3, 0),
            DataElement::Int128(_) => (2, 4, 0),
            DataElement::Uuid16(_) => (3, 1, 0),
            DataElement::Uuid32(_) => (3, 2, 0),
            DataElement::Uuid128(_) => (3, 4, 0),
            DataElement::String(s) => (4, 0, s.len()),
            DataElement::Bool(_) => (5, 0, 0),
            DataElement::Sequence(s) => (6, 0, s.len()),
            DataElement::Alternative(s) => (7, 0, s.len()),
            DataElement::Url(s) => (8, 0, s.len()),
        };

        if size_desc == 0 && size != 0 {
            if size < u8::MAX as usize {
                size_desc = 5;
            } else if size < u16::MAX as usize {
                size_desc = 6;
            } else if size < u32::MAX as usize {
                size_desc = 7;
            } else {
                panic!("size of data too large");
            }
        }

        let header = (type_desc << 5) | size_desc;

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

    fn into_u8(self) -> Option<u8> {
        match self {
            Self::Uint8(v) => Some(v),
            _ => None,
        }
    }

    fn into_u16(self) -> Option<u16> {
        match self {
            Self::Uint16(v) => Some(v),
            _ => None,
        }
    }

    fn into_u32(self) -> Option<u32> {
        match self {
            Self::Uint32(v) => Some(v),
            _ => None,
        }
    }

    fn into_u64(self) -> Option<u64> {
        match self {
            Self::Uint64(v) => Some(v),
            _ => None,
        }
    }

    fn into_u128(self) -> Option<u128> {
        match self {
            Self::Uint128(v) => Some(v),
            _ => None,
        }
    }

    fn into_sequence(self) -> Option<Vec<DataElement>> {
        match self {
            Self::Sequence(v) => Some(v),
            _ => None,
        }
    }
}

#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromPrimitive, ToPrimitive)]
pub enum ErrorCode {
    UnsupportedSdpVersion = 0x0001,
    InvalidServiceRecordHandle,
    InvalidRequestSyntax,
    InvalidPduSize,
    InvalidContinuationState,
    InsufficientResources,
}

struct ErrorResponse {
    code: ErrorCode,
}

impl<B: Buf> From<&mut B> for ErrorResponse {
    fn from(buf: &mut B) -> Self {
        Self {
            code: FromPrimitive::from_u8(buf.get_u8()).unwrap(),
        }
    }
}

struct ServiceSearchRequest {
    service_search_pattern: Vec<Uuid128>,
    maximum_service_record_count: u16,
    continuation_state: Vec<u8>,
}

impl ToBuf for ServiceSearchRequest {
    fn to_buf<B: BufMut>(&self, buf: &mut B) {
        let service_search_pat = DataElement::Sequence(
            self.service_search_pattern
                .iter()
                .map(|u| DataElement::Uuid128(*u))
                .collect(),
        );
        service_search_pat.to_buf(buf);
        buf.put_u16(self.maximum_service_record_count);
        buf.put_u8(self.continuation_state.len() as u8);
        buf.put(self.continuation_state.as_ref());
    }
}

pub struct ServiceSearchResponse {
    total_service_record_count: u16,
    current_service_record_count: u16,
    service_record_handle_list: Vec<u32>,
    continuation_state: Vec<u8>,
}

impl<B: Buf> From<&mut B> for ServiceSearchResponse {
    fn from(buf: &mut B) -> Self {
        Self {
            total_service_record_count: buf.get_u16(),
            current_service_record_count: buf.get_u16(),
            service_record_handle_list: DataElement::from(&mut *buf)
                .into_sequence()
                .unwrap()
                .into_iter()
                .map(|de| de.into_u32())
                .collect::<Option<Vec<u32>>>()
                .unwrap(),
            continuation_state: {
                let continuation_state_size = buf.get_u8();
                buf.get_vec_u8(continuation_state_size as usize)
            },
        }
    }
}

#[derive(Debug)]
pub struct SdpClient(BluetoothStream);

impl SdpClient {
    fn send(&mut self, req: Pdu) {
        let mut buf = BytesMut::new();
        req.to_buf(&mut buf);
        self.0.write_all(buf.as_ref()).unwrap();
    }

    fn recv(&mut self) -> Pdu {
        let mut buf = BytesMut::new();
        self.0.read(buf.as_mut()).unwrap();
        Pdu::from(&mut buf)
    }

    pub fn service_search_request(
        &mut self,
        service_search_pattern: Vec<Uuid128>,
        maximum_service_record_count: u16,
    ) -> ServiceSearchResponse {
        let mut continuation_state = Vec::new();
        let mut res: Option<ServiceSearchResponse> = None;

        loop {
            let req = ServiceSearchRequest {
                service_search_pattern: service_search_pattern.clone(),
                maximum_service_record_count,
                continuation_state: continuation_state.clone(),
            };
            let req_pdu = Pdu::with_parameter(PduId::ServiceSearchRequest, 0, req);
            self.send(req_pdu);

            let mut res_pdu = self.recv();
            match res_pdu.id {
                PduId::ErrorResponse => panic!("got error response"),
                PduId::ServiceSearchResponse => {
                    let new_res = ServiceSearchResponse::from(&mut res_pdu.parameter);

                    res = if let Some(mut res) = res {
                        res.service_record_handle_list
                            .extend(new_res.service_record_handle_list);

                        if res.continuation_state.len() == 0 {
                            break res;
                        } else {
                            continuation_state = res.continuation_state;
                            res.continuation_state = Vec::new();
                        }

                        Some(res)
                    } else {
                        Some(new_res)
                    }
                }
                _ => panic!("got wrong response to request"),
            }
        }
    }
}
