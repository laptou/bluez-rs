use bitvec::{field::BitField, prelude as bv, view::BitView};
use bytes::{Buf, Bytes};
use enumflags2::{bitflags, BitFlags};

#[bitflags]
#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ServiceClass {
    Positioning = 1 << 16,
    Networking = 1 << 17,
    Rendering = 1 << 18,
    Capturing = 1 << 19,
    ObjectTransfer = 1 << 20,
    Audio = 1 << 21,
    Telephony = 1 << 22,
    Information = 1 << 23,
}

pub type ServiceClasses = BitFlags<ServiceClass>;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum DeviceClass {
    Computer(ComputerDeviceClass),
    Phone(PhoneDeviceClass),

    /// The parameter is the amount of utilisation the access point currently has, expressed as a
    /// fraction.
    AccessPoint(f64),
    AudioVideo(AudioVideoDeviceClass),
    Peripheral {
        keyboard: bool,
        pointer: bool,
        class: PeripheralDeviceClass,
    },
    Imaging {
        display: bool,
        camera: bool,
        scanner: bool,
        printer: bool,
    },
    Wearable(WearableDeviceClass),
    Toy(ToyDeviceClass),
    Health(HealthDeviceClass),
    Uncategorized,
    Unknown,
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum ComputerDeviceClass {
    Uncategorized,
    Desktop,
    Server,
    Laptop,
    HandheldPDA,
    PalmPDA,
    Wearable,
    Tablet,
    Unknown,
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum PhoneDeviceClass {
    Uncategorized,
    Cellular,
    Cordless,
    Smartphone,
    Modem,
    ISDN,
    Unknown,
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum AudioVideoDeviceClass {
    Headset,
    HandsFree,
    Microphone,
    Loudspeaker,
    Headphones,
    Portable,
    Car,
    SetTop,
    HiFi,
    VCR,
    VideoCamera,
    Camcorder,
    VideoMonitor,
    VideoDisplayLoudspeaker,
    VideoConferencing,
    Gaming,
    Unknown,
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum PeripheralDeviceClass {
    Uncategorized,
    Joystick,
    Gamepad,
    Remote,
    Sensor,
    Digitizer,
    CardReader,
    Pen,
    Scanner,
    Wand,
    Unknown,
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum WearableDeviceClass {
    Wristwatch,
    Pager,
    Jacket,
    Helmet,
    Glasses,
    Unknown,
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum ToyDeviceClass {
    Robot,
    Vehicle,
    Doll,
    Controller,
    Game,
    Unknown,
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum HealthDeviceClass {
    BloodPressureMeter,
    Thermometer,
    WeightScale,
    GlucoseMeter,
    PulseOximeter,
    HeartRateMonitor,
    HealthDataDisplay,
    StepCounter,
    BodyCompositionAnalyzer,
    PeakFlowMonitor,
    MedicationMonitor,
    KneeProsthesis,
    AnkleProsthesis,
    GenericHealthManager,
    PersonalMobilityDevice,
    Unknown,
}

pub fn device_class_from_bytes(class: Bytes) -> (DeviceClass, ServiceClasses) {
    let bits = class[0] as u32 | ((class[1] as u32) << 8) | ((class[2] as u32) << 16);
    device_class_from_u32(bits)
}

pub fn device_class_from_buf<B: Buf>(class: &mut B) -> (DeviceClass, ServiceClasses) {
    let mut items = [0u8; 3];
    class.copy_to_slice(&mut items[..]);
    device_class_from_array(items)
}

pub fn device_class_from_array(class: [u8; 3]) -> (DeviceClass, ServiceClasses) {
    let bits = class[0] as u32 | ((class[1] as u32) << 8) | ((class[2] as u32) << 16);
    device_class_from_u32(bits)
}

pub fn device_class_from_u32(class: u32) -> (DeviceClass, ServiceClasses) {
    let service_classes = ServiceClasses::from_bits_truncate(class);

    let class_bits = class.view_bits::<bv::Lsb0>();

    // major device class encoded in bits 8-12
    let device_class = match class_bits[8..13].load::<u8>() {
        // minor device class in bits 2-7
        0b00001 => DeviceClass::Computer(match class_bits[2..8].load::<u8>() {
            0b000000 => ComputerDeviceClass::Uncategorized,
            0b000001 => ComputerDeviceClass::Desktop,
            0b000010 => ComputerDeviceClass::Server,
            0b000011 => ComputerDeviceClass::Laptop,
            0b000100 => ComputerDeviceClass::HandheldPDA,
            0b000101 => ComputerDeviceClass::PalmPDA,
            0b000110 => ComputerDeviceClass::Wearable,
            0b000111 => ComputerDeviceClass::Tablet,
            _ => ComputerDeviceClass::Unknown,
        }),
        0b00010 => DeviceClass::Phone(match class_bits[2..8].load::<u8>() {
            0b000000 => PhoneDeviceClass::Uncategorized,
            0b000001 => PhoneDeviceClass::Cellular,
            0b000010 => PhoneDeviceClass::Cordless,
            0b000011 => PhoneDeviceClass::Smartphone,
            0b000100 => PhoneDeviceClass::Modem,
            0b000101 => PhoneDeviceClass::ISDN,
            _ => PhoneDeviceClass::Unknown,
        }),
        0b00011 => DeviceClass::AccessPoint(0.),
        0b00100 => DeviceClass::AudioVideo(match class_bits[2..8].load::<u8>() {
            0b000001 => AudioVideoDeviceClass::Headset,
            0b000010 => AudioVideoDeviceClass::HandsFree,
            0b000011 => AudioVideoDeviceClass::Unknown,
            0b000100 => AudioVideoDeviceClass::Microphone,
            0b000101 => AudioVideoDeviceClass::Loudspeaker,
            0b000110 => AudioVideoDeviceClass::Headphones,
            0b000111 => AudioVideoDeviceClass::Portable,
            0b001000 => AudioVideoDeviceClass::Car,
            0b001001 => AudioVideoDeviceClass::SetTop,
            0b001010 => AudioVideoDeviceClass::HiFi,
            0b001011 => AudioVideoDeviceClass::VCR,
            0b001100 => AudioVideoDeviceClass::VideoCamera,
            0b001101 => AudioVideoDeviceClass::Camcorder,
            0b001110 => AudioVideoDeviceClass::VideoMonitor,
            0b001111 => AudioVideoDeviceClass::VideoDisplayLoudspeaker,
            0b010000 => AudioVideoDeviceClass::VideoConferencing,
            0b010001 => AudioVideoDeviceClass::Unknown,
            0b010010 => AudioVideoDeviceClass::Gaming,
            _ => AudioVideoDeviceClass::Unknown,
        }),
        0b00101 => DeviceClass::Peripheral {
            keyboard: class_bits[6],
            pointer: class_bits[7],
            class: match class_bits[2..6].load::<u8>() {
                0b0000 => PeripheralDeviceClass::Uncategorized,
                0b0001 => PeripheralDeviceClass::Joystick,
                0b0010 => PeripheralDeviceClass::Gamepad,
                0b0011 => PeripheralDeviceClass::Remote,
                0b0100 => PeripheralDeviceClass::Sensor,
                0b0101 => PeripheralDeviceClass::Digitizer,
                0b0110 => PeripheralDeviceClass::CardReader,
                0b0111 => PeripheralDeviceClass::Pen,
                0b1000 => PeripheralDeviceClass::Scanner,
                0b1001 => PeripheralDeviceClass::Wand,
                _ => PeripheralDeviceClass::Unknown,
            },
        },
        0b00110 => DeviceClass::Imaging {
            display: class_bits[4],
            camera: class_bits[5],
            scanner: class_bits[6],
            printer: class_bits[7],
        },
        0b00111 => DeviceClass::Wearable(match class_bits[2..8].load::<u8>() {
            0b0001 => WearableDeviceClass::Wristwatch,
            0b0010 => WearableDeviceClass::Pager,
            0b0011 => WearableDeviceClass::Jacket,
            0b0100 => WearableDeviceClass::Helmet,
            0b0101 => WearableDeviceClass::Glasses,
            _ => WearableDeviceClass::Unknown,
        }),
        0b01000 => DeviceClass::Toy(match class_bits[2..8].load::<u8>() {
            0b0001 => ToyDeviceClass::Robot,
            0b0010 => ToyDeviceClass::Vehicle,
            0b0011 => ToyDeviceClass::Doll,
            0b0100 => ToyDeviceClass::Controller,
            0b0101 => ToyDeviceClass::Game,
            _ => ToyDeviceClass::Unknown,
        }),
        0b01001 => DeviceClass::Health(match class_bits[2..8].load::<u8>() {
            0b000001 => HealthDeviceClass::BloodPressureMeter,
            0b000010 => HealthDeviceClass::Thermometer,
            0b000011 => HealthDeviceClass::WeightScale,
            0b000100 => HealthDeviceClass::GlucoseMeter,
            0b000101 => HealthDeviceClass::PulseOximeter,
            0b000110 => HealthDeviceClass::HeartRateMonitor,
            0b000111 => HealthDeviceClass::HealthDataDisplay,
            0b001000 => HealthDeviceClass::StepCounter,
            0b001001 => HealthDeviceClass::BodyCompositionAnalyzer,
            0b001010 => HealthDeviceClass::PeakFlowMonitor,
            0b001011 => HealthDeviceClass::MedicationMonitor,
            0b001100 => HealthDeviceClass::KneeProsthesis,
            0b001101 => HealthDeviceClass::AnkleProsthesis,
            0b001110 => HealthDeviceClass::GenericHealthManager,
            0b001111 => HealthDeviceClass::PersonalMobilityDevice,
            _ => HealthDeviceClass::Unknown,
        }),
        0b11111 => DeviceClass::Uncategorized,
        _ => DeviceClass::Unknown,
    };

    (device_class, service_classes)
}

impl From<DeviceClass> for u16 {
    fn from(val: DeviceClass) -> Self {
        let mut bits = 0u16;

        match val {
            DeviceClass::Computer(minor) => {
                bits |= 0b00001 << 8;
                match minor {
                    ComputerDeviceClass::Desktop => bits |= 0b000001 << 2,
                    ComputerDeviceClass::Server => bits |= 0b000010 << 2,
                    ComputerDeviceClass::Laptop => bits |= 0b000011 << 2,
                    ComputerDeviceClass::HandheldPDA => bits |= 0b000100 << 2,
                    ComputerDeviceClass::PalmPDA => bits |= 0b000101 << 2,
                    ComputerDeviceClass::Wearable => bits |= 0b000110 << 2,
                    ComputerDeviceClass::Tablet => bits |= 0b000111 << 2,
                    _ => (),
                }
            }
            DeviceClass::Phone(minor) => {
                bits |= 0b00010 << 8;
                match minor {
                    PhoneDeviceClass::Cellular => bits |= 0b000001 << 2,
                    PhoneDeviceClass::Cordless => bits |= 0b000010 << 2,
                    PhoneDeviceClass::Smartphone => bits |= 0b000011 << 2,
                    PhoneDeviceClass::Modem => bits |= 0b000100 << 2,
                    PhoneDeviceClass::ISDN => bits |= 0b000101 << 2,
                    _ => (),
                }
            }
            DeviceClass::AccessPoint(..) => {
                // bits |= 0b00011 << 8;
                unimplemented!()
            }
            DeviceClass::AudioVideo(minor) => {
                bits |= 0b00100 << 8;
                match minor {
                    AudioVideoDeviceClass::Headset => bits |= 0b000001 << 2,
                    AudioVideoDeviceClass::HandsFree => bits |= 0b000010 << 2,
                    // 000011 is reserved
                    AudioVideoDeviceClass::Microphone => bits |= 0b000100 << 2,
                    AudioVideoDeviceClass::Loudspeaker => bits |= 0b000101 << 2,
                    AudioVideoDeviceClass::Headphones => bits |= 0b000110 << 2,
                    AudioVideoDeviceClass::Portable => bits |= 0b000111 << 2,
                    AudioVideoDeviceClass::Car => bits |= 0b001000 << 2,
                    AudioVideoDeviceClass::SetTop => bits |= 0b001001 << 2,
                    AudioVideoDeviceClass::HiFi => bits |= 0b001010 << 2,
                    AudioVideoDeviceClass::VCR => bits |= 0b001011 << 2,
                    AudioVideoDeviceClass::VideoCamera => bits |= 0b001100 << 2,
                    AudioVideoDeviceClass::Camcorder => bits |= 0b001101 << 2,
                    AudioVideoDeviceClass::VideoMonitor => bits |= 0b001110 << 2,
                    AudioVideoDeviceClass::VideoDisplayLoudspeaker => bits |= 0b001111 << 2,
                    AudioVideoDeviceClass::VideoConferencing => bits |= 0b010000 << 2,
                    // 010001 is reserved
                    AudioVideoDeviceClass::Gaming => bits |= 0b010010 << 2,
                    _ => (),
                }
            }
            DeviceClass::Peripheral {
                keyboard,
                pointer,
                class,
            } => {
                bits |= 0b00101 << 8;
                if keyboard {
                    bits |= 1 << 6
                }
                if pointer {
                    bits |= 1 << 7
                }

                match class {
                    PeripheralDeviceClass::Joystick => bits |= 0b0001 << 2,
                    PeripheralDeviceClass::Gamepad => bits |= 0b0010 << 2,
                    PeripheralDeviceClass::Remote => bits |= 0b0011 << 2,
                    PeripheralDeviceClass::Sensor => bits |= 0b0100 << 2,
                    PeripheralDeviceClass::Digitizer => bits |= 0b0101 << 2,
                    PeripheralDeviceClass::CardReader => bits |= 0b0110 << 2,
                    PeripheralDeviceClass::Pen => bits |= 0b0111 << 2,
                    PeripheralDeviceClass::Scanner => bits |= 0b1000 << 2,
                    PeripheralDeviceClass::Wand => bits |= 0b1001 << 2,
                    _ => (),
                }
            }
            DeviceClass::Imaging {
                display,
                camera,
                scanner,
                printer,
            } => {
                bits |= 0b00110 << 8;

                if display {
                    bits |= 1 << 4
                }
                if camera {
                    bits |= 1 << 5
                }
                if scanner {
                    bits |= 1 << 6
                }
                if printer {
                    bits |= 1 << 7
                }
            }
            DeviceClass::Wearable(minor) => {
                bits |= 0b00111 << 8;

                match minor {
                    WearableDeviceClass::Wristwatch => bits |= 0b000001 << 2,
                    WearableDeviceClass::Pager => bits |= 0b000010 << 2,
                    WearableDeviceClass::Jacket => bits |= 0b000011 << 2,
                    WearableDeviceClass::Helmet => bits |= 0b000100 << 2,
                    WearableDeviceClass::Glasses => bits |= 0b000101 << 2,
                    _ => (),
                }
            }
            DeviceClass::Toy(minor) => {
                bits |= 0b01000 << 8;

                match minor {
                    ToyDeviceClass::Robot => bits |= 0b000001 << 2,
                    ToyDeviceClass::Vehicle => bits |= 0b000010 << 2,
                    ToyDeviceClass::Doll => bits |= 0b000011 << 2,
                    ToyDeviceClass::Controller => bits |= 0b000100 << 2,
                    ToyDeviceClass::Game => bits |= 0b000101 << 2,
                    _ => (),
                }
            }
            DeviceClass::Health(minor) => {
                bits |= 0b01001 << 8;

                match minor {
                    HealthDeviceClass::BloodPressureMeter => bits |= 0b000001 << 2,
                    HealthDeviceClass::Thermometer => bits |= 0b000010 << 2,
                    HealthDeviceClass::WeightScale => bits |= 0b000011 << 2,
                    HealthDeviceClass::GlucoseMeter => bits |= 0b000100 << 2,
                    HealthDeviceClass::PulseOximeter => bits |= 0b000101 << 2,
                    HealthDeviceClass::HeartRateMonitor => bits |= 0b000110 << 2,
                    HealthDeviceClass::HealthDataDisplay => bits |= 0b000111 << 2,
                    HealthDeviceClass::StepCounter => bits |= 0b001000 << 2,
                    HealthDeviceClass::BodyCompositionAnalyzer => bits |= 0b001001 << 2,
                    HealthDeviceClass::PeakFlowMonitor => bits |= 0b001010 << 2,
                    HealthDeviceClass::MedicationMonitor => bits |= 0b001011 << 2,
                    HealthDeviceClass::KneeProsthesis => bits |= 0b001100 << 2,
                    HealthDeviceClass::AnkleProsthesis => bits |= 0b001101 << 2,
                    HealthDeviceClass::GenericHealthManager => bits |= 0b001110 << 2,
                    HealthDeviceClass::PersonalMobilityDevice => bits |= 0b001111 << 2,
                    _ => (),
                }
            }
            DeviceClass::Uncategorized => {
                bits |= 0b11111 << 8;
            }
            DeviceClass::Unknown => (),
        }

        bits
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn class() {
        let c = DeviceClass::Computer(ComputerDeviceClass::Laptop);
        let b: u16 = c.into();
        println!("{:000000000000b} (0x{:x})", b, b);
        let (c1, _) = device_class_from_u32(b as u32);
        assert_eq!(c, c1);
    }
}
