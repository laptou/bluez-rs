#[repr(u16)]
#[derive(Debug, Copy, Clone)]
pub enum BtProto {
    L2CAP = 0,
    HCI = 1,
    RFCOMM = 3,
    AVDTP = 7,
}
