use std::ffi::OsString;
use std::fmt::{Display, Formatter};

use crate::Address;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Controller(pub(crate) u16);

impl Display for Controller {
    fn fmt(&self, f: &mut Formatter) -> Result<(), ::std::fmt::Error> {
        write!(f, "hci{}", self.0)
    }
}

impl Into<u16> for Controller {
    fn into(self) -> u16 {
        return self.0;
    }
}

impl Controller {
    pub fn none() -> Controller {
        Controller(0xFFFF)
    }
}

pub struct ControllerInfo {
    pub address: Address,
    pub bluetooth_version: u8,
    pub manufacturer: [u8; 2],
    pub supported_settings: ControllerSettings,
    pub current_settings: ControllerSettings,
    pub class_of_device: [u8; 3],
    pub name: OsString,
    pub short_name: OsString,
}

bitflags! {
    pub struct ControllerSettings: u32 {
        #[allow(non_upper_case_globals)] const Powered = 1 << 0;
        #[allow(non_upper_case_globals)] const Connectable = 1 << 1;
        #[allow(non_upper_case_globals)] const FastConnectable = 1 << 2;
        #[allow(non_upper_case_globals)] const Discoverable = 1 << 3;
        #[allow(non_upper_case_globals)] const Pairable = 1 << 4;
        #[allow(non_upper_case_globals)] const LinkLevelSecurity = 1 << 5;
        #[allow(non_upper_case_globals)] const SecureSimplePairing = 1 << 6;
        #[allow(non_upper_case_globals)] const BREDR = 1 << 7;
        #[allow(non_upper_case_globals)] const HighSpeed = 1 << 8;
        #[allow(non_upper_case_globals)] const LE = 1 << 9;
        #[allow(non_upper_case_globals)] const Advertising = 1 << 10;
        #[allow(non_upper_case_globals)] const SecureConnection = 1 << 11;
        #[allow(non_upper_case_globals)] const DebugKeys = 1 << 12;
        #[allow(non_upper_case_globals)] const Privacy = 1 << 13;
        #[allow(non_upper_case_globals)] const Configuration = 1 << 14;
        #[allow(non_upper_case_globals)] const StaticAddress = 1 << 15;
    }
}

impl Display for ControllerSettings {
    fn fmt(&self, f: &mut Formatter) -> Result<(), ::std::fmt::Error> {
        let mut tags = vec![];

        if self.contains(ControllerSettings::Powered) {
            tags.push("powered");
        }

        if self.contains(ControllerSettings::Connectable) {
            tags.push("connectable");
        }

        if self.contains(ControllerSettings::FastConnectable) {
            tags.push("fast-connectable");
        }

        if self.contains(ControllerSettings::Discoverable) {
            tags.push("discoverable");
        }

        if self.contains(ControllerSettings::Pairable) {
            tags.push("pairable");
        }

        if self.contains(ControllerSettings::LinkLevelSecurity) {
            tags.push("link-level-security");
        }

        if self.contains(ControllerSettings::SecureSimplePairing) {
            tags.push("secure-simple-pairing");
        }

        if self.contains(ControllerSettings::BREDR) {
            tags.push("br/edr");
        }

        if self.contains(ControllerSettings::HighSpeed) {
            tags.push("high-speed");
        }

        if self.contains(ControllerSettings::LE) {
            tags.push("low-energy");
        }

        if self.contains(ControllerSettings::Advertising) {
            tags.push("advertising");
        }

        if self.contains(ControllerSettings::SecureConnection) {
            tags.push("secure-connection");
        }

        if self.contains(ControllerSettings::DebugKeys) {
            tags.push("debug-keys");
        }

        if self.contains(ControllerSettings::Privacy) {
            tags.push("privacy");
        }

        if self.contains(ControllerSettings::StaticAddress) {
            tags.push("static-address");
        }

        write!(f, "{}", tags.join(" "))
    }
}
