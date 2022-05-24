use std::{collections::HashMap, fmt::Debug};

use super::{stream::BluetoothStream, Uuid};
use crate::address::Protocol;
use crate::util::BufExt;
use crate::{communication::Uuid16, Address, AddressType};
use error::{Error, ErrorCode};
use serialization::{DataElement, Pdu, PduId, ToBuf};

use bytes::{Buf, BufMut, BytesMut};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

mod error;
mod serialization;

pub const SDP_PSM: u16 = 0x0001;
pub const SDP_BROWSE_ROOT: Uuid16 = Uuid16(0x1002);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceAttributeRange {
    Single(ServiceAttributeId),
    Range(ServiceAttributeId, ServiceAttributeId),
}

impl ServiceAttributeRange {
    pub const ALL: Self = Self::Range(ServiceAttributeId(0), ServiceAttributeId(u16::MAX));
}

struct ServiceSearchRequest {
    service_search_pattern: Vec<Uuid>,
    maximum_service_record_count: u16,
    continuation_state: Vec<u8>,
}

impl ToBuf for ServiceSearchRequest {
    fn to_buf<B: BufMut>(&self, buf: &mut B) {
        let service_search_pat = DataElement::Sequence(
            self.service_search_pattern
                .iter()
                .map(|u| match *u {
                    Uuid::Uuid16(u) => DataElement::Uuid16(u),
                    Uuid::Uuid32(u) => DataElement::Uuid32(u),
                    Uuid::Uuid128(u) => DataElement::Uuid128(u),
                })
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
    pub service_record_handles: Vec<u32>,
    continuation_state: Vec<u8>,
}

impl<B: Buf> From<&mut B> for ServiceSearchResponse {
    fn from(buf: &mut B) -> Self {
        let _total_service_record_count = buf.get_u16();

        let current_service_record_count = buf.get_u16();

        Self {
            service_record_handles: (0..current_service_record_count)
                .map(|_| buf.get_u32())
                .collect(),

            continuation_state: {
                let continuation_state_size = buf.get_u8();
                buf.get_vec_u8(continuation_state_size as usize)
            },
        }
    }
}

struct ServiceAttributeRequest {
    service_handle: u32,
    maximum_attribute_byte_count: u16,
    attribute_id_list: Vec<ServiceAttributeRange>,
    continuation_state: Vec<u8>,
}

impl ToBuf for ServiceAttributeRequest {
    fn to_buf<B: BufMut>(&self, buf: &mut B) {
        buf.put_u32(self.service_handle);
        buf.put_u16(self.maximum_attribute_byte_count);

        let attribute_id_list = DataElement::Sequence(
            self.attribute_id_list
                .iter()
                .map(|range| match range {
                    &ServiceAttributeRange::Single(item) => DataElement::Uint16(item.0),
                    &ServiceAttributeRange::Range(start, end) => {
                        DataElement::Uint32(((start.0 as u32) << 16) | end.0 as u32)
                    }
                })
                .collect(),
        );

        attribute_id_list.to_buf(buf);

        buf.put_u8(self.continuation_state.len() as u8);
        buf.put(self.continuation_state.as_ref());
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Hash)]
pub struct ServiceAttributeId(pub u16);

impl Debug for ServiceAttributeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:04x?}", self.0)
    }
}

impl ServiceAttributeId {
    pub const SERVICE_RECORD_HANDLE: Self = Self(0x0000);
    pub const SERVICE_CLASS_ID_LIST: Self = Self(0x0001);
    pub const SERVICE_RECORD_STATE: Self = Self(0x0002);
    pub const SERVICE_ID: Self = Self(0x0003);
    pub const PROTOCOL_DESCRIPTOR_LIST: Self = Self(0x0004);
    pub const BROWSE_GROUP_LIST: Self = Self(0x0005);
    pub const LANGUAGE_BASE_ATTRIBUTE_ID_LIST: Self = Self(0x0006);
    pub const SERVICE_INFO_TIME_TO_LIVE: Self = Self(0x0007);
    pub const SERVICE_AVAILABILITY: Self = Self(0x0008);
    pub const BLUETOOTH_PROFILE_DESCRIPTOR_LIST: Self = Self(0x0009);
    pub const DOCUMENTATION_URL: Self = Self(0x000A);
    pub const CLIENT_EXECUTABLE_URL: Self = Self(0x000B);
    pub const ICON_URL: Self = Self(0x000C);
    pub const ADDITIONAL_PROTOCOL_DESCRIPTOR_LISTS: Self = Self(0x000D);
}

#[derive(Debug, Clone)]
pub struct ServiceAttributeResponse {
    pub attributes: HashMap<ServiceAttributeId, DataElement>,
    pub continuation_state: Vec<u8>,
}

impl<B: Buf> From<&mut B> for ServiceAttributeResponse {
    fn from(buf: &mut B) -> Self {
        let _attribute_byte_count = buf.get_u16();
        let attribute_list = DataElement::from(&mut *buf);

        if let DataElement::Sequence(attribute_list) = attribute_list {
            // println!("recv attr list: {:#?}", attribute_list);

            let mut attributes = HashMap::new();

            for pair in attribute_list.chunks_exact(2) {
                let attribute_id = if let &DataElement::Uint16(attribute_id) = &pair[0] {
                    attribute_id
                } else {
                    panic!("expected attribute id to be a u16");
                };

                attributes.insert(ServiceAttributeId(attribute_id), pair[1].clone());
            }

            return Self {
                attributes,
                continuation_state: {
                    let continuation_state_size = buf.get_u8();
                    buf.get_vec_u8(continuation_state_size as usize)
                },
            };
        } else {
            panic!("expected attribute list to be a sequence");
        }
    }
}

#[derive(Debug)]
pub struct ServiceDiscoveryClient(BluetoothStream);

impl ServiceDiscoveryClient {
    async fn send(&mut self, req: Pdu) -> Result<(), Error> {
        let mut buf = BytesMut::new();
        req.to_buf(&mut buf);
        // println!("send buf: {:02x?}", &buf[..]);
        self.0.write_all(buf.as_ref()).await?;
        Ok(())
    }

    async fn recv(&mut self) -> Result<Pdu, Error> {
        let mut buf = BytesMut::with_capacity(65536);
        self.0.read_buf(&mut buf).await?;
        // println!("recv buf: {:02x?}", &buf[..]);
        Ok(Pdu::from(&mut buf))
    }

    pub async fn connect(address: Address) -> Result<Self, Error> {
        let stream =
            BluetoothStream::connect(Protocol::L2CAP, address, AddressType::BREDR, SDP_PSM).await?;
        Ok(Self(stream))
    }

    pub async fn service_search(
        &mut self,
        service_search_pattern: Vec<Uuid>,
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
            self.send(req_pdu).await?;
            txn += 1;

            let mut res_pdu = self.recv().await?;
            match res_pdu.id {
                PduId::ErrorResponse => {
                    return Err(Error::Remote(ErrorCode::from(&mut res_pdu.parameter)))
                }
                PduId::ServiceSearchResponse => {
                    let new_res = ServiceSearchResponse::from(&mut res_pdu.parameter);

                    if let Some(res) = &mut res {
                        res.service_record_handles
                            .extend(new_res.service_record_handles);
                        res.continuation_state = new_res.continuation_state;
                    } else {
                        res = Some(new_res)
                    }

                    if res.as_ref().unwrap().continuation_state.len() == 0 {
                        break res.unwrap();
                    }
                }
                _ => return Err(Error::InvalidResponse),
            }
        })
    }

    pub async fn service_attribute(
        &mut self,
        service_handle: u32,
        maximum_attribute_byte_count: u16,
        attribute_id_list: Vec<ServiceAttributeRange>,
    ) -> Result<ServiceAttributeResponse, Error> {
        let mut res: Option<ServiceAttributeResponse> = None;
        let mut txn = 0;

        Ok(loop {
            let req = ServiceAttributeRequest {
                attribute_id_list: attribute_id_list.clone(),
                maximum_attribute_byte_count,
                service_handle,
                continuation_state: res
                    .as_ref()
                    .map(|r| r.continuation_state.clone())
                    .unwrap_or(vec![]),
            };

            let req_pdu = Pdu::with_parameter(PduId::ServiceAttributeRequest, txn, req);
            self.send(req_pdu).await?;
            txn += 1;

            let mut res_pdu = self.recv().await?;
            match res_pdu.id {
                PduId::ErrorResponse => {
                    return Err(Error::Remote(ErrorCode::from(&mut res_pdu.parameter)))
                }
                PduId::ServiceAttributeResponse => {
                    let new_res = ServiceAttributeResponse::from(&mut res_pdu.parameter);

                    if let Some(res) = &mut res {
                        res.attributes.extend(new_res.attributes);
                        res.continuation_state = new_res.continuation_state;
                    } else {
                        res = Some(new_res)
                    }

                    if res.as_ref().unwrap().continuation_state.len() == 0 {
                        break res.unwrap();
                    }
                }
                _ => return Err(Error::InvalidResponse),
            }
        })
    }
}
