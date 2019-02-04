use std::ptr;

use errno::{errno, Errno};
use failure::{Error, Fail};
use libc;

use crate::Address;
use crate::bt;

use super::device::Device;

#[derive(Debug)]
pub enum ServiceClass {
    Positioning,
    Networking,
    Rendering,
    Capturing,
    ObjectTransfer,
    Audio,
    Telephony,
    Information,
}

#[derive(Debug)]
pub enum DeviceClass {
    Miscellaneous,
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

#[derive(Debug)]
pub enum ComputerDeviceClass {
    Uncategorized,
    Desktop,
    Server,
    Laptop,
    HandheldPDA,
    ClamshellPDA,
    Wearable,
    Tablet,
    Unknown,
}

#[derive(Debug)]
pub enum PhoneDeviceClass {
    Uncategorized,
    Cellular,
    Cordless,
    Smartphone,
    Modem,
    ISDN,
    Unknown,
}

#[derive(Debug)]
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

#[derive(Debug)]
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

#[derive(Debug)]
pub enum WearableDeviceClass {
    Wristwatch,
    Pager,
    Jacket,
    Helmet,
    Glasses,
    Unknown,
}

#[derive(Debug)]
pub enum ToyDeviceClass {
    Robot,
    Vehicle,
    Doll,
    Controller,
    Game,
    Unknown,
}

#[derive(Debug)]
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

#[derive(Debug)]
pub struct InquiryResponse {
    pub address: Address,
    pub service_classes: Vec<ServiceClass>,
    pub device_class: DeviceClass,
}

#[derive(Debug, Fail)]
pub enum InquiryError {
    #[fail(display = "Could not query available devices: {}.", err)]
    CouldNotInquire { err: Errno },
}

/// Returns a list of nearby devices.
/// * `timeout` - The inquiry will take at most `timeout` * 1.28 seconds to complete.
/// * `max_responses` - The maximum number of responses for the inquiry to return.
/// * `flush` - Whether the cache should be flushed before performing this inquiry.
/// If this is false, devices that are no longer available may be returned.
pub fn inquire(
    device: &Device,
    timeout: i32,
    max_responses: i32,
    flush: bool,
) -> Result<Vec<InquiryResponse>, Error> {
    let flags = if flush {
        bt::IREQ_CACHE_FLUSH as i64
    } else {
        0
    };

    let mut ii = ptr::null_mut::<bt::inquiry_info>();

    let num = unsafe {
        bt::hci_inquiry(
            device.id,
            timeout,
            max_responses,
            ptr::null(),
            &mut ii,
            flags,
        )
    };

    if num < 0 {
        return Err(InquiryError::CouldNotInquire { err: errno() }.into());
    }

    return Ok((0..num)
        .map(|offset| {
            let info = unsafe { *ii.offset(offset as isize) };
            let device_class: DeviceClass;
            let mut service_classes: Vec<ServiceClass> = Vec::new();

            let bits = info.dev_class[0] as u64
                | ((info.dev_class[1] as u64) << 8)
                | ((info.dev_class[2] as u64) << 16);

            if bits & (1 << 16) == (1 << 16) {
                service_classes.push(ServiceClass::Positioning);
            }
            if bits & (1 << 17) == (1 << 17) {
                service_classes.push(ServiceClass::Networking);
            }
            if bits & (1 << 18) == (1 << 18) {
                service_classes.push(ServiceClass::Rendering);
            }
            if bits & (1 << 19) == (1 << 19) {
                service_classes.push(ServiceClass::Capturing);
            }
            if bits & (1 << 20) == (1 << 20) {
                service_classes.push(ServiceClass::ObjectTransfer);
            }
            if bits & (1 << 21) == (1 << 21) {
                service_classes.push(ServiceClass::Audio);
            }
            if bits & (1 << 22) == (1 << 22) {
                service_classes.push(ServiceClass::Telephony);
            }
            if bits & (1 << 23) == (1 << 23) {
                service_classes.push(ServiceClass::Information);
            }

            if bits & (0b00001 << 8) == (0b00001 << 8) {
                device_class = DeviceClass::Computer(if bits & (0b000000 << 2) == (0b000000 << 2) {
                    ComputerDeviceClass::Uncategorized
                } else if bits & (0b000001 << 2) == (0b000001 << 2) {
                    ComputerDeviceClass::Desktop
                } else if bits & (0b000010 << 2) == (0b000010 << 2) {
                    ComputerDeviceClass::Server
                } else if bits & (0b000011 << 2) == (0b000011 << 2) {
                    ComputerDeviceClass::Laptop
                } else if bits & (0b000100 << 2) == (0b000100 << 2) {
                    ComputerDeviceClass::HandheldPDA
                } else if bits & (0b000101 << 2) == (0b000101 << 2) {
                    ComputerDeviceClass::ClamshellPDA
                } else if bits & (0b000110 << 2) == (0b000110 << 2) {
                    ComputerDeviceClass::Wearable
                } else if bits & (0b000111 << 2) == (0b000111 << 2) {
                    ComputerDeviceClass::Tablet
                } else {
                    ComputerDeviceClass::Unknown
                });
            } else if bits & (0b00010 << 8) == (0b00010 << 8) {
                device_class = DeviceClass::Phone(if bits & (0b000000 << 2) == (0b000000 << 2) {
                    PhoneDeviceClass::Uncategorized
                } else if bits & (0b000001 << 2) == (0b000001 << 2) {
                    PhoneDeviceClass::Cellular
                } else if bits & (0b000010 << 2) == (0b000010 << 2) {
                    PhoneDeviceClass::Cordless
                } else if bits & (0b000011 << 2) == (0b000011 << 2) {
                    PhoneDeviceClass::Smartphone
                } else if bits & (0b000100 << 2) == (0b000100 << 2) {
                    PhoneDeviceClass::Modem
                } else if bits & (0b000101 << 2) == (0b000101 << 2) {
                    PhoneDeviceClass::ISDN
                } else {
                    PhoneDeviceClass::Unknown
                });
            } else if bits & (0b00011 << 8) == (0b00011 << 8) {
                device_class = DeviceClass::AccessPoint(0.);
            } else if bits & (0b00100 << 8) == (0b00100 << 8) {
                device_class =
                    DeviceClass::AudioVideo(if bits & (0b000001 << 2) == (0b000001 << 2) {
                        AudioVideoDeviceClass::Headset
                    } else if bits & (0b000010 << 2) == (0b000010 << 2) {
                        AudioVideoDeviceClass::HandsFree
                    } else if bits & (0b000011 << 2) == (0b000011 << 2) {
                        AudioVideoDeviceClass::Unknown
                    } else if bits & (0b000100 << 2) == (0b000100 << 2) {
                        AudioVideoDeviceClass::Microphone
                    } else if bits & (0b000101 << 2) == (0b000101 << 2) {
                        AudioVideoDeviceClass::Loudspeaker
                    } else if bits & (0b000110 << 2) == (0b000110 << 2) {
                        AudioVideoDeviceClass::Headphones
                    } else if bits & (0b000111 << 2) == (0b000111 << 2) {
                        AudioVideoDeviceClass::Portable
                    } else if bits & (0b001000 << 2) == (0b001000 << 2) {
                        AudioVideoDeviceClass::Car
                    } else if bits & (0b001001 << 2) == (0b001001 << 2) {
                        AudioVideoDeviceClass::SetTop
                    } else if bits & (0b001010 << 2) == (0b001010 << 2) {
                        AudioVideoDeviceClass::HiFi
                    } else if bits & (0b001011 << 2) == (0b001011 << 2) {
                        AudioVideoDeviceClass::VCR
                    } else if bits & (0b001100 << 2) == (0b001100 << 2) {
                        AudioVideoDeviceClass::VideoCamera
                    } else if bits & (0b001101 << 2) == (0b001101 << 2) {
                        AudioVideoDeviceClass::Camcorder
                    } else if bits & (0b001110 << 2) == (0b001110 << 2) {
                        AudioVideoDeviceClass::VideoMonitor
                    } else if bits & (0b001111 << 2) == (0b001111 << 2) {
                        AudioVideoDeviceClass::VideoDisplayLoudspeaker
                    } else if bits & (0b010000 << 2) == (0b010000 << 2) {
                        AudioVideoDeviceClass::VideoConferencing
                    } else if bits & (0b010001 << 2) == (0b010001 << 2) {
                        AudioVideoDeviceClass::Unknown
                    } else if bits & (0b010010 << 2) == (0b010010 << 2) {
                        AudioVideoDeviceClass::Gaming
                    } else {
                        AudioVideoDeviceClass::Unknown
                    });
            } else if bits & (0b00101 << 8) == (0b00101 << 8) {
                device_class = DeviceClass::Peripheral {
                    keyboard: bits & (1 << 6) == (1 << 6),
                    pointer: bits & (1 << 7) == (1 << 7),
                    class: if bits & (0b0000 << 2) == (0b0000 << 2) {
                        PeripheralDeviceClass::Uncategorized
                    } else if bits & (0b0001 << 2) == (0b0001 << 2) {
                        PeripheralDeviceClass::Joystick
                    } else if bits & (0b0010 << 2) == (0b0010 << 2) {
                        PeripheralDeviceClass::Gamepad
                    } else if bits & (0b0011 << 2) == (0b0011 << 2) {
                        PeripheralDeviceClass::Remote
                    } else if bits & (0b0100 << 2) == (0b0100 << 2) {
                        PeripheralDeviceClass::Sensor
                    } else if bits & (0b0101 << 2) == (0b0101 << 2) {
                        PeripheralDeviceClass::Digitizer
                    } else if bits & (0b0110 << 2) == (0b0110 << 2) {
                        PeripheralDeviceClass::CardReader
                    } else if bits & (0b0111 << 2) == (0b0111 << 2) {
                        PeripheralDeviceClass::Pen
                    } else if bits & (0b1000 << 2) == (0b1000 << 2) {
                        PeripheralDeviceClass::Scanner
                    } else if bits & (0b1001 << 2) == (0b1001 << 2) {
                        PeripheralDeviceClass::Wand
                    } else {
                        PeripheralDeviceClass::Unknown
                    },
                };
            } else if bits & (0b00110 << 8) == (0b00110 << 8) {
                device_class = DeviceClass::Imaging {
                    display: bits & (1 << 4) == (1 << 4),
                    camera: bits & (1 << 5) == (1 << 5),
                    scanner: bits & (1 << 6) == (1 << 6),
                    printer: bits & (1 << 7) == (1 << 7),
                }
            } else if bits & (0b00111 << 8) == (0b00111 << 8) {
                device_class = DeviceClass::Wearable(if bits & (0b0001 << 2) == (0b0001 << 2) {
                    WearableDeviceClass::Wristwatch
                } else if bits & (0b0010 << 2) == (0b0010 << 2) {
                    WearableDeviceClass::Pager
                } else if bits & (0b0011 << 2) == (0b0011 << 2) {
                    WearableDeviceClass::Jacket
                } else if bits & (0b0100 << 2) == (0b0100 << 2) {
                    WearableDeviceClass::Helmet
                } else if bits & (0b0101 << 2) == (0b0101 << 2) {
                    WearableDeviceClass::Glasses
                } else {
                    WearableDeviceClass::Unknown
                });
            } else if bits & (0b01000 << 8) == (0b01000 << 8) {
                device_class = DeviceClass::Toy(if bits & (0b0001 << 2) == (0b0001 << 2) {
                    ToyDeviceClass::Robot
                } else if bits & (0b0010 << 2) == (0b0010 << 2) {
                    ToyDeviceClass::Vehicle
                } else if bits & (0b0011 << 2) == (0b0011 << 2) {
                    ToyDeviceClass::Doll
                } else if bits & (0b0100 << 2) == (0b0100 << 2) {
                    ToyDeviceClass::Controller
                } else if bits & (0b0101 << 2) == (0b0101 << 2) {
                    ToyDeviceClass::Game
                } else {
                    ToyDeviceClass::Unknown
                });
            } else if bits & (0b01001 << 8) == (0b01001 << 8) {
                device_class = DeviceClass::Health(if bits & (0b000001 << 2) == (0b000001 << 2) {
                    HealthDeviceClass::BloodPressureMeter
                } else if bits & (0b000010 << 2) == (0b000010 << 2) {
                    HealthDeviceClass::Thermometer
                } else if bits & (0b000011 << 2) == (0b000011 << 2) {
                    HealthDeviceClass::WeightScale
                } else if bits & (0b000100 << 2) == (0b000100 << 2) {
                    HealthDeviceClass::GlucoseMeter
                } else if bits & (0b000101 << 2) == (0b000101 << 2) {
                    HealthDeviceClass::PulseOximeter
                } else if bits & (0b000110 << 2) == (0b000110 << 2) {
                    HealthDeviceClass::HeartRateMonitor
                } else if bits & (0b000111 << 2) == (0b000111 << 2) {
                    HealthDeviceClass::HealthDataDisplay
                } else if bits & (0b001000 << 2) == (0b001000 << 2) {
                    HealthDeviceClass::StepCounter
                } else if bits & (0b001001 << 2) == (0b001001 << 2) {
                    HealthDeviceClass::BodyCompositionAnalyzer
                } else if bits & (0b001010 << 2) == (0b001010 << 2) {
                    HealthDeviceClass::PeakFlowMonitor
                } else if bits & (0b001011 << 2) == (0b001011 << 2) {
                    HealthDeviceClass::MedicationMonitor
                } else if bits & (0b001100 << 2) == (0b001100 << 2) {
                    HealthDeviceClass::KneeProsthesis
                } else if bits & (0b001101 << 2) == (0b001101 << 2) {
                    HealthDeviceClass::AnkleProsthesis
                } else if bits & (0b001110 << 2) == (0b001110 << 2) {
                    HealthDeviceClass::GenericHealthManager
                } else if bits & (0b001111 << 2) == (0b001111 << 2) {
                    HealthDeviceClass::PersonalMobilityDevice
                } else {
                    HealthDeviceClass::Unknown
                });
            } else if bits & (0b11111 << 8) == (0b11111 << 8) {
                device_class = DeviceClass::Uncategorized;
            } else if bits & (0b00000 << 8) == (0b00000 << 8) {
                device_class = DeviceClass::Miscellaneous;
            } else {
                device_class = DeviceClass::Unknown;
            }

            let address = Address::from(info.bdaddr);

            core::mem::drop(ii);

            InquiryResponse {
                address,
                device_class,
                service_classes,
            }
        })
        .collect());
}
