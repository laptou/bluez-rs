use enumflags2::BitFlags;

use crate::Address;

// all of these structs are defined as packed structs here
// https://elixir.bootlin.com/linux/latest/source/include/net/bluetooth/mgmt.h
// I could just generate structs from the headers and rename them, but
// packed structs aren't ideal especially b/c things like strings, vectors, arrays
// aren't represented exactly the same in C and in Rust, and it would in general
// just make things kind of tricky, versus the current approach which might
// be a little slower and definitely more verbose but also nice and easy to use from
// the Rust side, which is the point of this library
// plus the Nomicon discourages it https://doc.rust-lang.org/nomicon/other-reprs.html

#[repr(u8)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, FromPrimitive)]
pub enum AddressType {
    BREDR = 0,
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
#[derive(Debug, Copy, Clone, Eq, PartialEq, BitFlags)]
pub enum AddressTypeFlag {
    BREDR = 1 << 0,
    LEPublic = 1 << 1,
    LERandom = 1 << 2,
}

#[derive(Debug)]
pub struct LinkKey {
    pub address: Address,
    pub address_type: AddressType,
    pub key_type: LinkKeyType,
    pub value: [u8; 16],
    pub pin_length: u8,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, FromPrimitive)]
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

#[derive(Debug, Copy, Clone, Eq, PartialEq, FromPrimitive)]
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
#[derive(Debug, Copy, Clone, BitFlags, Eq, PartialEq, FromPrimitive)]
pub enum DeviceFlags {
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
pub struct ConnectionParameters {
    pub address: Address,
    pub address_type: AddressType,
    pub min_connection_interval: u16,
    pub max_connection_interval: u16,
    pub connection_latency: u16,
    pub supervision_timeout: u16,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, BitFlags)]
pub enum ControllerConfigOptions {
    External = 1 << 0,
    BluetoothPublicAddr = 1 << 1,
}

#[derive(Debug)]
pub struct ControllerConfigInfo {
    pub manufacturer: [u8; 2],
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

pub struct AdvertisingFeaturesInfo {
    pub supported_flags: BitFlags<AdvertisingFlags>,
    pub max_adv_data_len: u8,
    pub max_scan_rsp_len: u8,
    pub max_instances: u8,
    pub instances: Vec<u8>,
}

pub struct AdvertisingSizeInfo {
    pub instance: u8,
    pub flags: BitFlags<AdvertisingFlags>,
    pub max_adv_data_len: u8,
    pub max_scan_rsp_len: u8,
}

pub struct AdvertisingParams {
    pub instance: u8,

    ///	When the `EnterConnectable` flag is not set, then the controller will
    ///	use advertising based on the connectable setting. When using
    ///	non-connectable or scannable advertising, the controller will
    ///	be programmed with a non-resolvable random address. When the
    ///	system is connectable, then the identity address or resolvable
    ///	private address will be used.
    ///
    ///	Using the `EnterConnectable` flag is useful for peripheral mode support
    ///	where BR/EDR (and/or LE) is controlled by Add Device. This allows
    ///	making the peripheral connectable without having to interfere
    ///	with the global connectable setting.
    pub flags: BitFlags<AdvertisingFlags>,

    /// Configures the length of an Instance. The
    ///	value is in seconds.
    ///
    ///	A value of 0 indicates a default value is chosen for the
    ///	`duration`. The default is 2 seconds.
    pub duration: u16,

    /// Configures the life-time of an Instance. In
    ///	case the value 0 is used it indicates no expiration time. If a
    ///	timeout value is provided, then the advertising Instance will be
    ///	automatically removed when the timeout passes. The value for the
    ///	timeout is in seconds. Powering down a controller will invalidate
    ///	all advertising Instances and it is not possible to add a new
    ///	Instance with a timeout when the controller is powered down.
    pub timeout: u16,
    pub adv_data: Vec<u8>,

    ///	If `scan_rsp` is empty and connectable flag is not set and
    ///	the global connectable setting is off, then non-connectable
    ///	advertising is used. If `scan_rsp` is not empty
    ///	connectable flag is not set and the global advertising is off,
    ///	then scannable advertising is used. This small difference is
    ///	supported to provide less air traffic for devices implementing
    ///	broadcaster role.
    pub scan_rsp: Vec<u8>,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, BitFlags)]
pub enum AdvertisingFlags {
    /// Indicates support for connectable advertising
    ///	and for switching to connectable advertising independent of the
    ///	connectable global setting. When this flag is not supported, then
    ///	the global connectable setting determines if undirected connectable,
    ///	undirected scannable or undirected non-connectable advertising is
    ///	used. It also determines the use of non-resolvable random address
    ///	versus identity address or resolvable private address.
    EnterConnectable = 1 << 0,

    /// Indicates support for advertising with discoverable
    ///	mode enabled. Users of this flag will decrease the `max_adv_data_len`
    ///	by 3. In this case the advertising data flags are managed
    ///	and added in front of the provided advertising data.
    AdvertiseDiscoverable = 1 << 1,

    /// Indicates support for advertising with limited
    ///	discoverable mode enabled. Users of this flag will decrease the
    ///	`max_adv_data_len` by 3. In this case the advertising data
    ///	flags are managed and added in front of the provided advertising
    ///	data.
    AdvertiseLimitedDiscoverable = 1 << 2,

    /// Indicates support for automatically keeping the
    ///	Flags field of the advertising data updated. Users of this flag
    ///	will decrease the `max_adv_data_len` by 3 and need to keep
    ///	that in mind. The Flags field will be added in front of the
    ///	advertising data provided by the user. Note that with `AdvertiseDiscoverable`
    /// and `AdvertiseLimitedDiscoverable`, this one will be implicitly used even if it is
    ///	not marked as supported.
    AutoUpdateFlags = 1 << 3,

    /// Indicates support for automatically adding the
    ///	TX Power value to the advertising data. Users of this flag will
    ///	decrease the `max_adv_data_len` by 3. The `tx_power` field will
    ///	be added at the end of the user provided advertising data. If the
    ///	controller does not support TX Power information, then this bit will
    ///	not be set.
    AutoUpdateTxPower = 1 << 4,

    /// indicates support for automatically adding the
    ///	Appearance value to the scan response data. Users of this flag
    ///	will decrease the `max_scan_rsp_len` by 4. The `appearance`
    ///	field will be added in front of the scan response data provided
    ///	by the user. If the appearance value is not supported, then this
    ///	bit will not be set.
    AutoUpdateAppearance = 1 << 5,

    /// Indicates support for automatically adding the
    ///	Local Name value to the scan response data. This flag indicates
    ///	an opportunistic approach for the Local Name. If enough space
    ///	in the scan response data is available, it will be added. If the
    ///	space is limited a short version or no name information. The
    ///	Local Name will be added at the end of the scan response data.
    AutoUpdateLocalName = 1 << 6,

    /// Indicates support for advertising in secondary channel in LE 1M PHY.
    SecondaryChannelLE1M = 1 << 7,

    /// Indicates support for advertising in secondary channel in LE 2M PHY.
    /// Primary channel would be on 1M.
    SecondaryChannelLE2M = 1 << 8,

    /// Indicates support for advertising in secondary channel in LE CODED PHY.
    SecondaryChannelLECoded = 1 << 9,
}

pub struct PhyConfig {
    pub supported_phys: BitFlags<Phy>,
    pub configurable_phys: BitFlags<Phy>,
    pub selected_phys: BitFlags<Phy>,
}

#[repr(u32)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, BitFlags)]
pub enum Phy {
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

pub struct BlockedKey {
    pub key_type: BlockedKeyType,
    pub value: [u8; 16],
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, FromPrimitive)]
pub enum BlockedKeyType {
    LinkKey = 1 << 0,
    LongTermKey = 1 << 1,
    IdentityResolvingKey = 1 << 2,
}
