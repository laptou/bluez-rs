use enumflags2::BitFlags;

use crate::Address;

#[repr(u8)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, FromPrimitive)]
pub enum AddressType {
    BrEdr = 0,
    LEPublic = 1,
    LERandom = 2,
}

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
#[derive(Debug, Copy, Clone, Eq, PartialEq, FromPrimitive)]
pub enum DiscoveryAddressTypes {
    /// BR/EDR
    BrEdr = 1,
    /// LE (public & random)
    LE = 6,
    /// BR/EDR/LE (interleaved discovery)
    BrEdrLE = 7,
}

#[derive(Debug)]
pub struct LinkKey {
    pub address: Address,
    pub address_type: AddressType,
    pub key_type: LinkKeyType,
    pub value: [u8; 16],
    pub pin_length: u8,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u8)]
pub enum LinkKeyType {
    Combination = 0x00,
    LocalUnit = 0x01,
    RemoteUnit = 0x02,
    DebugCombination = 0x03,
    UnauthenticatedCombinationP192 = 0x04,
    AuthenticatedCombinationP192 = 0x05,
    ChangedCombination = 0x06,
    UnauthenticatedCombinationP256 = 0x07,
    AuthenticatedCombinationP256 = 0x08,
}

#[derive(Debug)]
pub struct LongTermKey {
    pub address: Address,
    pub address_type: AddressType,
    pub key_type: LongTermKeyType,
    pub master: u8,
    pub encryption_size: u8,
    pub encryption_diversifier: u16,
    pub random_number: u64,
    pub value: [u8; 16],
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u8)]
pub enum LongTermKeyType {
    Unauthenticated = 0x00,
    Authenticated = 0x01,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u8)]
pub enum IoCapability {
    DisplayOnly = 1,
    DisplayYesNo,
    KeyboardOnly,
    NoInputNoOutput,
    KeyboardDisplay,
}

#[derive(Debug)]
pub struct OutOfBandData {
    pub hash_192: [u8; 16],
    pub randomizer_192: [u8; 16],
    pub hash_256: Option<[u8; 16]>,
    pub randomizer_256: Option<[u8; 16]>,
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
pub struct IdentityResolvingKey {
    pub address: Address,
    pub address_type: AddressType,
    pub value: [u8; 16],
}

#[derive(Debug)]
pub struct ConnectionInfo {
    pub address: Address,
    pub address_type: AddressType,
    pub rssi: Option<u8>,
    pub tx_power: Option<u8>,
    pub max_tx_power: Option<u8>,
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
#[derive(Debug, Copy, Clone, BitFlags, Eq, PartialEq)]
pub enum DeviceFlags {
    ConfirmName = 1 << 0,
    LegacyPairing = 1 << 1
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum DisconnectionReason {
    Unspecified = 0,
    Timeout = 1,
    TerminatedLocal = 2,
    TerminatedRemote = 3
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum AddDeviceAction {
    BackgroundScan = 0,
    AllowConnect = 1,
    AutoConnect = 2,
}

#[derive(Debug)]
pub struct ConnectionParameters {
    pub address: Address,
    pub address_type: AddressType,
    pub min_connection_interval: u16,
    pub max_connection_interval: u16,
    pub connection_latency: u16,
    pub supervision_timeout: u16,
}


