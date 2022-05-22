use std::{
    ffi::OsString,
    fmt::Debug,
    io::{Read, Write},
    os::unix::prelude::{OsStrExt, OsStringExt},
};

use bytes::{Buf, BufMut, Bytes, BytesMut};
use num_traits::FromPrimitive;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use crate::communication::{Uuid128, Uuid16, Uuid32};

use crate::util::BufExtBlueZ;

use super::stream::BluetoothStream;

pub const SDP_PSM: u16 = 0x0001;

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
    fn to_buf<B: BufMut>(&self, buf: &mut B) {
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

#[derive(Error, Debug)]
pub enum Error {
    #[error("an i/o error occurred")]
    Io(#[from] std::io::Error),

    #[error("the remote device returned an error: {0:?}")]
    Remote(ErrorCode),
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

impl<B: Buf> From<&mut B> for ErrorCode {
    fn from(buf: &mut B) -> Self {
        let code = buf.get_u16();
        FromPrimitive::from_u16(code).unwrap()
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

#[derive(Debug, Clone)]
pub struct ServiceSearchResponse {
    pub total_service_record_count: u16,
    pub current_service_record_count: u16,
    pub service_record_handle_list: Vec<u32>,
    pub continuation_state: Vec<u8>,
}

impl<B: Buf> From<&mut B> for ServiceSearchResponse {
    fn from(buf: &mut B) -> Self {
        let total_service_record_count = buf.get_u16();
        let current_service_record_count = buf.get_u16();
        Self {
            total_service_record_count,
            current_service_record_count,
            service_record_handle_list: (0..current_service_record_count)
                .map(|_| buf.get_u32())
                .collect(),
            continuation_state: {
                let continuation_state_size = buf.get_u8();
                buf.get_vec_u8(continuation_state_size as usize)
            },
        }
    }
}

async fn sdp_send(bs: &mut BluetoothStream, req: Pdu) -> Result<(), Error> {
    let mut buf = BytesMut::new();
    req.to_buf(&mut buf);
    println!("send buf: {:02x?}", &buf[..]);
    bs.write_all(buf.as_ref()).await?;
    Ok(())
}

async fn sdp_recv(bs: &mut BluetoothStream) -> Result<Pdu, Error> {
    let mut buf = BytesMut::with_capacity(65536);
    bs.read_buf(&mut buf).await?;
    println!("recv buf: {:02x?}", &buf[..]);
    Ok(Pdu::from(&mut buf))
}

pub async fn service_search_request(
    bs: &mut BluetoothStream,
    service_search_pattern: Vec<Uuid128>,
    maximum_service_record_count: u16,
) -> Result<ServiceSearchResponse, Error> {
    let mut res: Option<ServiceSearchResponse> = None;
    let mut txn = 0;

    Ok(loop {
        let req = ServiceSearchRequest {
            service_search_pattern: service_search_pattern.clone(),
            maximum_service_record_count,
            continuation_state: res
                .as_ref()
                .map(|r| r.continuation_state.clone())
                .unwrap_or(vec![]),
        };
        let req_pdu = Pdu::with_parameter(PduId::ServiceSearchRequest, txn, req);
        sdp_send(bs, req_pdu).await?;
        txn += 1;

        let mut res_pdu = sdp_recv(bs).await?;
        match res_pdu.id {
            PduId::ErrorResponse => {
                return Err(Error::Remote(ErrorCode::from(&mut res_pdu.parameter)))
            }
            PduId::ServiceSearchResponse => {
                let new_res = ServiceSearchResponse::from(&mut res_pdu.parameter);

                if let Some(res) = &mut res {
                    res.service_record_handle_list
                        .extend(new_res.service_record_handle_list);
                    res.continuation_state = new_res.continuation_state;
                } else {
                    res = Some(new_res)
                }

                if res.as_ref().unwrap().continuation_state.len() == 0 {
                    break res.unwrap();
                }
            }
            _ => panic!("got wrong response to request"),
        }
    })
}
