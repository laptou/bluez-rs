#[repr(u32)]
#[derive(Debug, Copy, Clone)]
pub enum BtProto {
    L2CAP = bluetooth_sys::BTPROTO_L2CAP,
    HCI = bluetooth_sys::BTPROTO_HCI,
    RFCOMM = bluetooth_sys::BTPROTO_RFCOMM,
}
