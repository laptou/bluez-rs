use std::hash::Hash;

use enumflags2::{bitflags, BitFlags};

use crate::{Address, AddressType};

// all of these structs are defined as packed structs here
// https://elixir.bootlin.com/linux/latest/source/include/net/bluetooth/mgmt.h
// I could just generate structs from the headers and rename them, but
// packed structs aren't ideal especially b/c things like strings, vectors, arrays
// aren't represented exactly the same in C and in Rust, and it would in general
// just make things kind of tricky, versus the current approach which might
// be a little slower and definitely more verbose but also nice and easy to use from
// the Rust side, which is the point of this library
// plus the Nomicon discourages it https://doc.rust-lang.org/nomicon/other-reprs.html

/// Used to represent the version of the BlueZ management
/// interface that is in use.
pub struct ManagementVersion {
    pub version: u8,
    pub revision: u16,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum DebugKeysMode {
    Discard = 0,
    Persist = 1,
    PersistAndGenerate = 2,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum SecureConnectionsMode {
    Disabled = 0,
    Enabled = 1,
    Only = 2,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum LeAdvertisingMode {
    Disabled = 0,
    WithConnectable = 1,
    Enabled = 2,
}

#[repr(u8)]
#[bitflags]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum AddressTypeFlag {
    BREDR = 1 << 0,
    LEPublic = 1 << 1,
    LERandom = 1 << 2,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u8)]
pub enum IoCapability {
    DisplayOnly = 0,
    DisplayYesNo,
    KeyboardOnly,
    NoInputNoOutput,
    KeyboardDisplay,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u8)]
pub enum DiscoverableMode {
    None = 0x00,
    General = 0x01,
    Limited = 0x02,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u8)]
pub enum PrivacyMode {
    Disabled = 0x00,
    Strict = 0x01,
    Limited = 0x02,
}

#[derive(Debug)]
pub struct ConnectionInfo {
    pub address: Address,
    pub address_type: AddressType,
    pub rssi: Option<i8>,
    pub tx_power: Option<i8>,
    pub max_tx_power: Option<i8>,
}

#[derive(Debug)]
pub struct ClockInfo {
    pub address: Address,
    pub address_type: AddressType,
    pub local_clock: u32,
    pub piconet_clock: Option<u32>,
    pub accuracy: Option<u16>,
}

#[repr(u32)]
#[bitflags]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum DeviceFlag {
    ConfirmName = 1 << 0,
    LegacyPairing = 1 << 1,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, FromPrimitive)]
pub enum DisconnectionReason {
    Unspecified = 0,
    Timeout = 1,
    TerminatedLocal = 2,
    TerminatedRemote = 3,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, FromPrimitive)]
pub enum AddDeviceAction {
    BackgroundScan = 0,
    AllowConnect = 1,
    AutoConnect = 2,
}

#[derive(Debug)]
pub struct ConnectionParams {
    pub address: Address,
    pub address_type: AddressType,
    pub min_connection_interval: u16,
    pub max_connection_interval: u16,
    pub connection_latency: u16,
    pub supervision_timeout: u16,
}

#[repr(u32)]
#[bitflags]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ControllerConfigOptions {
    External = 1 << 0,
    BluetoothPublicAddr = 1 << 1,
}

#[derive(Debug)]
pub struct ControllerConfigInfo {
    pub manufacturer: u16,
    pub supported_options: BitFlags<ControllerConfigOptions>,
    pub missing_options: BitFlags<ControllerConfigOptions>,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, FromPrimitive)]
#[repr(u8)]
pub enum ControllerType {
    Primary = 0x00,
    Unconfigured = 0x01,
    AlternateMacPhy = 0x02,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, FromPrimitive)]
#[repr(u8)]
pub enum ControllerBus {
    Virtual = 0x00,
    USB = 0x01,
    PCMCIA = 0x02,
    UART,
    RS232,
    PCI,
    SDIO,
    SPI,
    I2C,
    SMD,
}

pub struct PhyConfig {
    pub supported_phys: BitFlags<PhyFlag>,
    pub configurable_phys: BitFlags<PhyFlag>,
    pub selected_phys: BitFlags<PhyFlag>,
}

#[repr(u32)]
#[bitflags]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum PhyFlag {
    BR1M1Slot = 1 << 0,
    BR1M3Slot = 1 << 1,
    BR1M5Slot = 1 << 2,
    EDR2M1Slot = 1 << 3,
    EDR2M3Slot = 1 << 4,
    EDR2M5Slot = 1 << 5,
    EDR3M1Slot = 1 << 6,
    EDR3M3Slot = 1 << 7,
    EDR3M5Slot = 1 << 8,
    LE1MTx = 1 << 9,
    LE1MRx = 1 << 10,
    LE2MTx = 1 << 11,
    LE2MRx = 1 << 12,
    LECodedTx = 1 << 13,
    LECodedRx = 1 << 14,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, FromPrimitive)]
#[repr(u16)]
pub enum SystemConfigParameterType {
    BREDRPageScanType = 0x0000,
    BREDRPageScanInterval,
    BREDRPageScanWindow,
    BREDRInquiryScanType,
    BREDRInquiryScanInterval,
    BREDRInquiryScanWindow,
    BREDRLinkSupervisionTimeout,
    BREDRPageTimeout,
    BREDRMinSniffInterval,
    BREDRMaxSniffInterval,
    LEAdvertisementMinInterval,
    LEAdvertisementMaxInterval,
    LEMultiAdvertisementRotationInterval,
    LEScanningIntervalForAutoConnect,
    LEScanningWindowForAutoConnect,
    LEScanningIntervalForWakeScenarios,
    LEScanningWindowForWakeScenarios,
    LEScanningIntervalForDiscovery,
    LEScanningWindowForDiscovery,
    LEScanningIntervalForAdvMonitoring,
    LEScanningWindowForAdvMonitoring,
    LEScanningIntervalForConnect,
    LEScanningWindowForConnect,
    LEMinConnectionInterval,
    LEMaxConnectionInterval,
    LEConnectionLatency,
    LEConnectionSupervisionTimeout,
    LEAutoconnectTimeout,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, FromPrimitive)]
//#[repr(u16)] once there are known variants
#[non_exhaustive]
pub enum RuntimeConfigParameterType {}
