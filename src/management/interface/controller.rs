use std::ffi::CString;
use std::fmt::{Display, Formatter};

use bytes::Bytes;
use enumflags2::{bitflags, BitFlags};

use crate::management::interface::class::{DeviceClass, ServiceClasses};
use crate::Address;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Controller(pub(crate) u16);

impl Display for Controller {
    fn fmt(&self, f: &mut Formatter) -> Result<(), ::std::fmt::Error> {
        write!(f, "hci{}", self.0)
    }
}

impl From<Controller> for u16 {
    fn from(val: Controller) -> Self {
        val.0
    }
}

impl Controller {
    pub fn none() -> Controller {
        Controller(0xFFFF)
    }
}

#[derive(Debug)]
pub struct ControllerInfo {
    pub address: Address,
    pub bluetooth_version: u8,
    pub manufacturer: u16,
    pub supported_settings: ControllerSettings,
    pub current_settings: ControllerSettings,
    pub class_of_device: (DeviceClass, ServiceClasses),
    pub name: CString,
    pub short_name: CString,
}

pub struct ControllerInfoExt {
    pub address: Address,
    pub bluetooth_version: u8,
    pub manufacturer: u16,
    pub supported_settings: ControllerSettings,
    pub current_settings: ControllerSettings,

    /// Contains information about class of device,
    /// local name and other values. Not all of them might be present. For
    /// example a Low Energy only device does not contain class of device
    /// information.
    ///
    /// When any of the values in the `eir_data` field changes, the event
    /// Extended Controller Information Changed will be used to inform
    /// clients about the updated information.
    pub eir_data: Bytes,
}

#[bitflags]
#[derive(Copy, Clone, Debug, PartialEq)]
#[repr(u32)]
pub enum ControllerSetting {
    Powered = 1 << 0,
    Connectable = 1 << 1,
    FastConnectable = 1 << 2,
    Discoverable = 1 << 3,
    Pairable = 1 << 4,
    LinkLevelSecurity = 1 << 5,
    SecureSimplePairing = 1 << 6,
    BREDR = 1 << 7,
    HighSpeed = 1 << 8,
    LE = 1 << 9,
    Advertising = 1 << 10,
    SecureConnection = 1 << 11,
    DebugKeys = 1 << 12,
    Privacy = 1 << 13,
    Configuration = 1 << 14,
    StaticAddress = 1 << 15,
    PhyConfiguration = 1 << 16,
    WidebandSpeech = 1 << 17,
}

pub type ControllerSettings = BitFlags<ControllerSetting>;
