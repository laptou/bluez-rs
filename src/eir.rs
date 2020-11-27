use bytes::*;
use bytes::buf::BufExt;
use enumflags2::BitFlags;
use num_derive::FromPrimitive;    
use num_traits::FromPrimitive;
use thiserror::Error;

#[repr(u8)]
#[derive(Debug, Copy, Clone, BitFlags, Eq, PartialEq)]
pub enum EIRFlags {
    LELimitedDiscoverableMode  = 1 << 0,
    LEGeneralDiscoverableMode = 1 << 1,
    BREDRNotSupported = 1 << 2,
    ControllerSimultaneousLEBREDR = 1 << 3,
    HostSimultaneousLEBREDR = 1 << 4,
}

#[derive(Debug)]
pub struct EIRName {
    name: String,
    complete: bool,
}

impl EIRName {
    fn short_name(name: String) -> Self {
        EIRName {
            name: name,
            complete: false,
        }
    }
    fn complete_name(name: String) -> Self {
        EIRName {
            name: name,
            complete: true,
        }
    }
}

#[derive(Debug)]
pub struct ManufacturerSpecificData {
    company_identifier_code: u16,
    data: Bytes,
}

#[derive(Debug)]
pub struct EIR {
    flags: Option<BitFlags<EIRFlags>>,
    uuid16: Vec<u16>,
    uuid32: Vec<u32>,
    uuid128: Vec<u128>,
    name: Option<EIRName>,
    tx_power_level: Vec<i8>,
    uri: Vec<String>,
    manufacturer_specific_data: Vec<ManufacturerSpecificData>,
}

impl EIR {
    fn new() -> Self {
        EIR {
            flags: None,
            uuid16: Vec::new(),
            uuid32: Vec::new(),
            uuid128: Vec::new(),
            name: None,
            tx_power_level: Vec::new(),
            uri: Vec::new(),
            manufacturer_specific_data: Vec::new(),
        }
    }
}

#[derive(Error, Debug)]
pub enum EIRError {
    #[error("More than one flag block found.")]
    RepeatedFlag,
    #[error("More than one name block found.")]
    RepeatedName,
    #[error("Unexpected data length {}.", len)]
    UnexpectedDataLength {
        len: usize,
    },
    #[error("UTF-8 encoding error in URI.")]
    InvalidURI,
}

#[repr(u8)]
#[derive(FromPrimitive)]
enum EIRDataTypes {
    Flags = 0x01,
    UUID16Incomplete = 0x02,
    UUID16Complete = 0x03,
    UUID32Incomplete = 0x04,
    UUID32Complete = 0x05,
    UUID128Incomplete = 0x06,
    UUID128Complete = 0x07,
    NameShort = 0x08,
    NameComplete = 0x09,
    TxPowerLevel = 0x0A,
    URI = 0x24,
    ManufacturerSpecificData = 0xFF,
}

pub fn parse_eir<T: Buf>(mut buf: T) -> Result<EIR, EIRError> {
    let mut eir = EIR::new();

    while buf.has_remaining() {
        // Bluetooth Specification Version 5.2, Vol 3, part C, 8 EXTENDED INQUIRY RESPONSE DATA FORMAT
        // [EIRStructure0, EIRStructure1, ..., EIRStructureN, 0...]
        // EIRStructure:
        //  -- 1 octet --  -- Length octets --
        // [  Length     ,     Data           ]
        // Data:
        //  -- n octet --  -- Length - n octets --
        // [ EIRDataType , EIRData                ]
        let len = buf.get_u8();
        if len == 0 {
            break;
        }

        // data types are all 1 octet
        let data_type = buf.get_u8();
        let mut data = buf.take((len - 1).into());

        // Core Specification Supplement
        // EIRDataType values https://www.bluetooth.com/specifications/assigned-numbers/generic-access-profile/
        match FromPrimitive::from_u8(data_type) {
            Some(EIRDataTypes::Flags) => { // Flags
                if eir.flags.is_some() {
                    return Err(EIRError::RepeatedFlag);
                }
                eir.flags = Some(BitFlags::from_bits_truncate(data.get_u8()));
            },
            Some(EIRDataTypes::UUID16Incomplete)|Some(EIRDataTypes::UUID16Complete) => {
                if data.remaining() % 2 != 0 {
                    return Err(EIRError::UnexpectedDataLength { len:data.remaining() });
                }
                while data.has_remaining() {
                    eir.uuid16.push(data.get_u16_le());
                }
            },
            Some(EIRDataTypes::UUID32Incomplete)|Some(EIRDataTypes::UUID32Complete) => {
                if data.remaining() % 4 != 0 {
                    return Err(EIRError::UnexpectedDataLength { len:data.remaining() });
                }
                while data.has_remaining() {
                    eir.uuid32.push(data.get_u32_le());
                }
            },
            Some(EIRDataTypes::UUID128Incomplete)|Some(EIRDataTypes::UUID128Complete) => {
                if data.remaining() % 16 != 0 {
                    return Err(EIRError::UnexpectedDataLength { len:data.remaining() });                    
                }
                while data.has_remaining() {
                    eir.uuid128.push(data.get_u128_le());
                }
            },
            Some(EIRDataTypes::NameShort) => {
                if eir.name.is_some() {
                    return Err(EIRError::RepeatedName);
                }
                eir.name = Some(EIRName::short_name(String::from_utf8_lossy(data.bytes()).to_string()));
            },
            Some(EIRDataTypes::NameComplete) => {
                if eir.name.is_some() {
                    return Err(EIRError::RepeatedName);
                }
                eir.name = Some(EIRName::complete_name(String::from_utf8_lossy(data.bytes()).to_string()));
            },
            Some(EIRDataTypes::TxPowerLevel) => {
                eir.tx_power_level.push(data.get_i8());
            },
            Some(EIRDataTypes::URI) => {
                let uri_scheme = data.get_u8();
                if uri_scheme == 0x01 {
                    let uri = String::from_utf8(data.bytes().to_vec());
                    if uri.is_err() {
                        return Err(EIRError::InvalidURI);
                    }
                    eir.uri.push(uri.unwrap());
                } else {
                    // TODO: URI scheme translation. Skip for now.
                }
            },
            Some(EIRDataTypes::ManufacturerSpecificData) => {
                if data.remaining() < 2 {
                    return Err(EIRError::UnexpectedDataLength { len:data.remaining() });
                }
                eir.manufacturer_specific_data.push(
                    ManufacturerSpecificData {
                        company_identifier_code: data.get_u16_le(),
                        data: Bytes::copy_from_slice(data.bytes()),
                    }
                );
            },
            _ => {
                // Skip unknown data
            },
        }
        data.advance(data.remaining());
        buf = data.into_inner();
    }

    Ok(eir)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn eir_name_test() {
        let input = Bytes::copy_from_slice(b"\x04\x09ABC");
        let eir = parse_eir(input);
        assert!(eir.is_ok());
        let eir = eir.unwrap();
        assert!(eir.flags.is_none());
        assert!(eir.uuid16.is_empty());
        assert!(eir.uuid32.is_empty());
        assert!(eir.uuid128.is_empty());
        assert!(eir.name.is_some());
        let name = eir.name.unwrap();
        assert!(name.complete);
        assert_eq!(name.name, "ABC");
        assert!(eir.tx_power_level.is_empty());
        assert!(eir.uri.is_empty());
        assert!(eir.manufacturer_specific_data.is_empty());
    }

    #[test]
    pub fn eir_multiple_test() {
        let input = Bytes::copy_from_slice(b"\x02\x01\x06\x03\x03\xAB\xAC\x03\x08Hi");
        let eir = parse_eir(input);
        assert!(eir.is_ok());
        let eir = eir.unwrap();
        assert!(eir.flags.is_some());
        let flags = eir.flags.unwrap();
        assert_eq!(flags, EIRFlags::BREDRNotSupported | EIRFlags::LEGeneralDiscoverableMode);
        assert!(!eir.uuid16.is_empty());
        assert_eq!(eir.uuid16, vec![0xACAB]);
        assert!(eir.uuid32.is_empty());
        assert!(eir.uuid128.is_empty());
        assert!(eir.name.is_some());
        let name = eir.name.unwrap();
        assert!(!name.complete);
        assert_eq!(name.name, "Hi");
        assert!(eir.tx_power_level.is_empty());
        assert!(eir.uri.is_empty());
        assert!(eir.manufacturer_specific_data.is_empty());
    }
}
