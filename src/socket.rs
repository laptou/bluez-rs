#[repr(u32)]
#[derive(Debug, Copy, Clone)]
pub enum BtProto {
    L2CAP = bluez_sys::BTPROTO_L2CAP,
    HCI = bluez_sys::BTPROTO_HCI,
    RFCOMM = bluez_sys::BTPROTO_RFCOMM,
}
