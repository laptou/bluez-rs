use libc::{c_ushort, c_int};

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct SockAddrHci {
    pub hci_family: c_ushort,
    pub hci_dev: c_ushort,
    pub hci_channel: HciChannel,
}

#[repr(u16)]
#[allow(dead_code)]
#[derive(Debug, Copy, Clone)]
pub enum BtProto {
    L2CAP = 0,
    HCI = 1,
    RFCOMM = 3,
    AVDTP = 7,
}

pub const SOL_L2CAP: c_int = 6;

#[repr(u16)]
#[allow(dead_code)]
#[derive(Debug, Copy, Clone)]
pub enum HciChannel {
    Raw = 0,
    User = 1,
    Monitor = 2,
    Control = 3,
}

pub const HCI_DEV_NONE: c_ushort = 65535;
