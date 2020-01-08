use std::fmt::Formatter;

#[repr(u8)]
#[derive(FromPrimitive, ToPrimitive, Copy, Clone, Debug)]
pub enum ManagementCommandStatus {
    Success = 0x00,
    UnknownCommand = 0x01,
    NotConnected = 0x02,
    Failed = 0x03,
    ConnectFailed = 0x04,
    AuthenticationFailed = 0x05,
    NotPaired = 0x06,
    NoResources = 0x07,
    Timeout = 0x08,
    AlreadyConnected = 0x09,
    Busy = 0x0A,
    Rejected = 0x0B,
    NotSupported = 0x0C,
    InvalidParams = 0x0D,
    Disconnected = 0x0E,
    NotPowered = 0x0F,
    Cancelled = 0x10,
    InvalidIndex = 0x11,
    RFKilled = 0x12,
    AlreadyPaired = 0x13,
    PermissionDenied = 0x14,
}

#[repr(u16)]
#[derive(Eq, PartialEq, FromPrimitive, ToPrimitive, Copy, Clone, Debug)]
pub enum ManagementCommand {
    ReadVersionInfo = 0x0001,
    ReadSupportedCommands,
    ReadControllerIndexList,
    ReadControllerInfo,
    SetPowered,
    SetDiscoverable,
    SetConnectable,
    SetFastConnectable,
    SetPairable,
    SetLinkSecurity,
    SetSecureSimplePairing,
    SetHighSpeed,
    SetLowEnergy,
    SetDeviceClass,
    SetLocalName,
    AddUUID,
    RemoveUUID,
    LoadLinkKeys,
    LoadLongTermKeys,
    Disconnect,
    GetConnections,
    PinCodeReply,
    PinCodeNegativeReply,
    SetIOCapability,
    PairDevice,
    CancelPairDevice,
    UnpairDevice,
    UserConfirmationReply,
    UserConfirmationNegativeReply,
    UserPasskeyReply,
    UserPasskeyNegativeReply,
    ReadLocalOutOfBand,
    AddRemoteOutOfBand,
    RemoveRemoteOutOfBand,
    StartDiscovery,
    StopDiscovery,
    ConfirmName,
    BlockDevice,
    UnblockDevice,
    SetDeviceID,
    SetAdvertising,
    SetBREDR,
    SetStaticAddress,
    SetScanParameters,
}

impl ::std::fmt::LowerHex for ManagementCommandStatus {
    fn fmt(&self, f: &mut Formatter) -> Result<(), ::std::fmt::Error> {
        write!(f, "{:x}", *self as u8)
    }
}

#[repr(u8)]
pub enum Discoverability {
    None = 0x00,
    General = 0x01,
    Limited = 0x02,
}
