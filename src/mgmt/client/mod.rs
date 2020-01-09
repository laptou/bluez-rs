use std::convert::TryInto;
use std::ffi::CString;

use bytes::*;
use num_traits::FromPrimitive;

pub use params::*;
pub use settings::*;

use crate::Address;
use crate::mgmt::{ManagementError, Result};
use crate::mgmt::interface::{
    class, ManagementCommand, ManagementCommandStatus, ManagementRequest,
};
use crate::mgmt::interface::class::{DeviceClass, ServiceClasses};
use crate::mgmt::interface::controller::{Controller, ControllerInfo, ControllerSettings};
use crate::mgmt::interface::event::ManagementEvent;
use crate::mgmt::socket::ManagementSocket;

mod address;
mod params;
mod settings;

fn bytes_to_c_str(bytes: Bytes) -> CString {
    let iterator = bytes.into_iter();
    let bytes = iterator.take_while(|&i| i != 0).collect();
    return unsafe { CString::from_vec_unchecked(bytes) };
}

pub struct ManagementClient {
    socket: ManagementSocket,
}

impl ManagementClient {
    pub fn new() -> Self {
        // todo: fix that unwrap()
        ManagementClient {
            socket: ManagementSocket::open().unwrap(),
        }
    }

    #[inline]
    async fn exec_command<F: FnOnce(Controller, Option<Bytes>) -> Result<T>, T>(
        &mut self,
        opcode: ManagementCommand,
        controller: Controller,
        param: Option<Bytes>,
        callback: F,
    ) -> Result<T> {
        let param = param.unwrap_or(Bytes::new());

        // send request
        self.socket
            .send(ManagementRequest {
                opcode,
                controller,
                param,
            })
            .await?;

        // loop until we receive a relevant response
        // which is either command complete or command status
        // with the same opcode as the command that we sent
        loop {
            let response = self.socket.receive().await?;

            // if we got an error, just send that back to the user
            // otherwise, give the data received to our callback fn
            match response.event {
                ManagementEvent::CommandComplete {
                    status,
                    param,
                    opcode: evt_opcode,
                } if opcode == evt_opcode => {
                    return match status {
                        ManagementCommandStatus::Success => {
                            callback(response.controller, Some(param))
                        }
                        _ => Err(ManagementError::CommandError { opcode, status }),
                    }
                }
                ManagementEvent::CommandStatus {
                    status,
                    opcode: evt_opcode,
                } if opcode == evt_opcode => {
                    return match status {
                        ManagementCommandStatus::Success => callback(response.controller, None),
                        _ => Err(ManagementError::CommandError { opcode, status }),
                    }
                }
                _ => (),
            }
        }
    }

    /// This command returns the Management version and revision.
    ///	Besides, being informational the information can be used to
    ///	determine whether certain behavior has changed or bugs fixed
    ///	when interacting with the kernel.
    pub async fn get_mgmt_version(&mut self) -> Result<ManagementVersion> {
        self.exec_command(
            ManagementCommand::ReadVersionInfo,
            Controller::none(),
            None,
            |_, param| {
                let mut param = param.unwrap();
                Ok(ManagementVersion {
                    version: param.get_u8(),
                    revision: param.get_u16_le(),
                })
            },
        )
            .await
    }

    /// This command returns the list of currently known controllers.
    ///	Controllers added or removed after calling this command can be
    ///	monitored using the Index Added and Index Removed events.
    pub async fn get_controller_list(&mut self) -> Result<Vec<Controller>> {
        self.exec_command(
            ManagementCommand::ReadControllerIndexList,
            Controller::none(),
            None,
            |_, param| {
                let mut param = param.unwrap();
                let count = param.get_u16_le() as usize;
                let mut controllers = vec![Controller::none(); count];
                for i in 0..count {
                    controllers[i] = Controller(param.get_u16_le());
                }

                Ok(controllers)
            },
        )
            .await
    }

    /// This command is used to retrieve the current state and basic
    ///	information of a controller. It is typically used right after
    ///	getting the response to the Read Controller Index List command
    ///	or an Index Added event.
    ///
    ///	The `address` parameter describes the controllers public address
    ///	and it can be expected that it is set. However in case of single
    ///	mode Low Energy only controllers it can be `00:00:00:00:00:00`. To
    ///	power on the controller in this case, it is required to configure
    ///	a static address using Set Static `address` command first.
    ///
    ///	If the public address is set, then it will be used as identity
    ///	address for the controller. If no public address is available,
    ///	then the configured static address will be used as identity
    ///	address.
    ///
    ///	In the case of a dual-mode controller with public address that
    ///	is configured as Low Energy only device (BR/EDR switched off),
    ///	the static address is used when set and public address otherwise.
    ///
    ///	If no short name is set the Short_Name parameter will be all zeroes.
    pub async fn get_controller_info(&mut self, controller: Controller) -> Result<ControllerInfo> {
        self.exec_command(
            ManagementCommand::ReadControllerInfo,
            controller,
            None,
            |_, param| {
                let mut param = param.unwrap();

                Ok(ControllerInfo {
                    address: Address::from_slice(param.split_to(6).as_ref()),
                    bluetooth_version: param.get_u8(),
                    manufacturer: param.split_to(2).as_ref().try_into().unwrap(),
                    supported_settings: ControllerSettings::from_bits_truncate(param.get_u32_le()),
                    current_settings: ControllerSettings::from_bits_truncate(param.get_u32_le()),
                    class_of_device: class::from_bytes(param.split_to(3).to_bytes()),
                    name: bytes_to_c_str(param.split_to(249)),
                    short_name: bytes_to_c_str(param),
                })
            },
        )
            .await
    }

    /// This command is used to set the major and minor device class for
    ///	BR/EDR capable controllers.
    ///
    ///	This command will also implicitly disable caching of pending CoD
    ///	and EIR updates.
    ///
    ///	This command is only available for BR/EDR capable controllers
    ///	(e.g. not for single-mode LE ones).
    ///
    ///	This command can be used when the controller is not powered and
    ///	all settings will be programmed once powered.
    ///
    ///	In case the controller is powered off, Unknown will be returned
    ///	for the class of device parameter. And after power on the new
    ///	value will be announced via class of device changed event.
    pub async fn set_device_class(
        &mut self,
        controller: Controller,
        device_class: DeviceClass,
    ) -> Result<(DeviceClass, ServiceClasses)> {
        let mut param = BytesMut::with_capacity(2);
        param.put_u16_le(device_class.into());

        self.exec_command(
            ManagementCommand::SetDeviceClass,
            controller,
            Some(param.to_bytes()),
            |_, param| Ok(class::from_bytes(param.unwrap())),
        )
            .await
    }

    /// This command is used to set the local name of a controller. The
    ///	command parameters also include a short name which will be used
    ///	in case the full name doesn't fit within EIR/AD data.
    ///
    /// Name can be at most 248 bytes. Short name can be at most 10 bytes.
    /// This function returns a pair of OsStrings in the order (name, short_name).
    ///
    ///	This command can be used when the controller is not powered and
    ///	all settings will be programmed once powered.
    ///
    ///	The values of name and short name will be remembered when
    ///	switching the controller off and back on again. So the name
    ///	and short name only have to be set once when a new controller
    ///	is found and will stay until removed.
    pub async fn set_local_name(
        &mut self,
        controller: Controller,
        name: &str,
        short_name: Option<&str>,
    ) -> Result<(CString, CString)> {
        if name.len() > 248 {
            return Err(ManagementError::NameTooLong {
                name: name.to_owned(),
                max_len: 248,
            });
        }

        if let Some(short_name) = short_name {
            if short_name.len() > 10 {
                return Err(ManagementError::NameTooLong {
                    name: short_name.to_owned(),
                    max_len: 10,
                });
            }
        }

        let mut param = BytesMut::with_capacity(260);
        param.resize(260, 0); // initialize w/ zeros

        CString::new(name)?
            .as_bytes_with_nul()
            .copy_to_slice(&mut param[..249]);
        CString::new(short_name.unwrap_or(""))?
            .as_bytes_with_nul()
            .copy_to_slice(&mut param[249..]);

        self.exec_command(
            ManagementCommand::SetLocalName,
            controller,
            Some(param.to_bytes()),
            |_, param| {
                let mut param = param.unwrap();

                Ok((
                    CString::new(param.split_to(249).to_vec()).unwrap(),
                    CString::new(param.to_vec()).unwrap(),
                ))
            },
        )
            .await
    }

    ///	This command is used to add a UUID to be published in EIR data.
    ///	The accompanied SVC_Hint parameter is used to tell the kernel
    ///	whether the service class bits of the Class of Device value need
    ///	modifying due to this UUID.
    ///
    ///	This command can be used when the controller is not powered and
    ///	all settings will be programmed once powered.
    ///
    ///	In case the controller is powered off, 0x000000 will be returned
    ///	for the class of device parameter. And after power on the new
    ///	value will be announced via class of device changed event.
    pub async fn add_uuid(
        &mut self,
        controller: Controller,
        uuid: [u8; 16],
        svc_hint: ServiceClasses,
    ) -> Result<(DeviceClass, ServiceClasses)> {
        let mut param = BytesMut::with_capacity(17);
        param.put_slice(&uuid[..]);
        param.put_u8((svc_hint.bits() >> 16) as u8);

        self.exec_command(
            ManagementCommand::AddUUID,
            controller,
            Some(param.to_bytes()),
            |_, param| Ok(class::from_bytes(param.unwrap())),
        )
            .await
    }

    ///	This command is used to remove a UUID previously added using the
    ///	Add UUID command.
    ///
    ///	When the UUID parameter is an empty UUID (16 x 0x00), then all
    ///	previously loaded UUIDs will be removed.
    ///
    ///	This command can be used when the controller is not powered and
    ///	all settings will be programmed once powered.
    ///
    ///	In case the controller is powered off, 0x000000 will be returned
    ///	for the class of device parameter. And after power on the new
    ///	value will be announced via class of device changed event.
    pub async fn remove_uuid(
        &mut self,
        controller: Controller,
        uuid: [u8; 16],
    ) -> Result<(DeviceClass, ServiceClasses)> {
        let mut param = BytesMut::from(&uuid[..]);

        self.exec_command(
            ManagementCommand::RemoveUUID,
            controller,
            Some(param.to_bytes()),
            |_, param| Ok(class::from_bytes(param.unwrap())),
        )
            .await
    }

    /// This command is used to feed the kernel with currently known
    ///	link keys. The command does not need to be called again upon the
    ///	receipt of New Link Key events since the kernel updates its list
    ///	automatically.
    ///
    ///	The debug parameter is used to tell the kernel whether to
    ///	accept the usage of debug keys or not. The allowed values for
    ///	this parameter are 0x00 and 0x01. All other values will return
    ///	an Invalid Parameters response.
    ///
    ///	Usage of the debug parameter is deprecated and has been
    ///	replaced with the Set Debug Keys command. When setting the
    ///	debug option via Load Link Keys command it has the same
    ///	affect as setting it via Set Debug Keys and applies to all
    ///	keys in the system.
    pub async fn load_link_keys(
        &mut self,
        controller: Controller,
        keys: Vec<LinkKey>,
        debug: bool,
    ) -> Result<()> {
        let mut param = BytesMut::with_capacity(3 + keys.len() * 25);
        param.put_u8(debug as u8);
        param.put_u16_le(keys.len() as u16);

        for key in keys {
            param.put_slice(key.address.as_ref());
            param.put_u8(key.address_type as u8);
            param.put_u8(key.key_type as u8);
            param.put_slice(&key.value[..]);
            param.put_u8(key.pin_length);
        }

        self.exec_command(
            ManagementCommand::LoadLinkKeys,
            controller,
            Some(param.to_bytes()),
            |_, _| Ok(()),
        )
            .await
    }

    ///	This command is used to feed the kernel with currently known
    ///	(SMP) Long Term Keys. The command does not need to be called
    ///	again upon the receipt of New Long Term Key events since the
    ///	kernel updates its list automatically.
    ///
    ///	The provided address and address_type are the identity of
    ///	a device. So either its public address or static random address.
    ///
    ///	Unresolvable random addresses and resolvable random addresses are
    ///	not valid and will be rejected.
    ///
    ///	This command can be used when the controller is not powered.
    pub async fn load_long_term_keys(
        &mut self,
        controller: Controller,
        keys: Vec<LongTermKey>,
    ) -> Result<()> {
        let mut param = BytesMut::with_capacity(2 + keys.len() * 25);
        param.put_u16_le(keys.len() as u16);

        for key in keys {
            param.put_slice(key.address.as_ref());
            param.put_u8(key.address_type as u8);
            param.put_u8(key.key_type as u8);
            param.put_u8(key.master);
            param.put_u8(key.encryption_size);
            param.put_u16_le(key.encryption_diversifier);
            param.put_u64_le(key.random_number);
            param.put_slice(&key.value[..]);
        }

        self.exec_command(
            ManagementCommand::LoadLongTermKeys,
            controller,
            Some(param.to_bytes()),
            |_, _| Ok(()),
        )
            .await
    }

    ///	This command is used to set the IO Capability used for pairing.
    ///	The command accepts both SSP and SMP values.
    ///
    ///	Passing KeyboardDisplay will cause the kernel to
    ///	convert it to DisplayYesNo)in the case of a BR/EDR
    ///	connection (as KeyboardDisplay is specific to SMP).
    ///
    ///	This command can be used when the controller is not powered.
    pub async fn set_io_capability(
        &mut self,
        controller: Controller,
        io_capability: IoCapability,
    ) -> Result<()> {
        let mut param = BytesMut::with_capacity(1);
        param.put_u8(io_capability as u8);

        self.exec_command(
            ManagementCommand::SetIOCapability,
            controller,
            Some(param.to_bytes()),
            |_, _| Ok(()),
        )
            .await
    }

    /// This command is used to read the local Out of Band data.
    ///
    ///	This command can only be used when the controller is powered.
    ///
    ///	If Secure Connections support is enabled, then this command
    ///	will return P-192 versions of hash and randomizer as well as
    ///	P-256 versions of both.
    ///
    ///	Values returned by this command become invalid when the controller
    ///	is powered down. After each power-cycle it is required to call
    ///	this command again to get updated values.
    pub async fn read_local_oob_data(&mut self, controller: Controller) -> Result<OutOfBandData> {
        self.exec_command(
            ManagementCommand::ReadLocalOutOfBand,
            controller,
            None,
            |_, param| {
                let mut param = param.unwrap();
                Ok(OutOfBandData {
                    hash_192: param.split_to(16).as_ref().try_into().unwrap(),
                    randomizer_192: param.split_to(16).as_ref().try_into().unwrap(),
                    hash_256: if param.has_remaining() {
                        Some(param.split_to(16).as_ref().try_into().unwrap())
                    } else {
                        None
                    },
                    randomizer_256: if param.has_remaining() {
                        Some(param.split_to(16).as_ref().try_into().unwrap())
                    } else {
                        None
                    },
                })
            },
        )
            .await
    }

    ///	This command is used to start the process of discovering remote
    ///	devices. A Device Found event will be sent for each discovered
    ///	device.
    ///
    ///	Possible values for the `address_type` parameter are a bit-wise or
    ///	of the following bits:
    ///
    ///		0	BR/EDR
    ///		1	LE Public
    ///		2	LE Random
    ///
    ///	By combining these e.g. the following values are possible:
    ///
    ///		1	BR/EDR
    ///		6	LE (public & random)
    ///		7	BR/EDR/LE (interleaved discovery)
    ///
    ///	This command can only be used when the controller is powered.
    pub async fn start_discovery(
        &mut self,
        controller: Controller,
        address_types: DiscoveryAddressTypes,
    ) -> Result<DiscoveryAddressTypes> {
        let mut param = BytesMut::with_capacity(1);
        param.put_u8(address_types as u8);

        self.exec_command(
            ManagementCommand::StartDiscovery,
            controller,
            Some(param.to_bytes()),
            |_, param| Ok(FromPrimitive::from_u8(param.unwrap().get_u8()).unwrap()),
        )
            .await
    }

    /// This command is used to stop the discovery process started using
    ///	the Start Discovery command.
    ///
    ///	This command can only be used when the controller is powered.
    pub async fn stop_discovery(
        &mut self,
        controller: Controller,
        address_types: DiscoveryAddressTypes,
    ) -> Result<DiscoveryAddressTypes> {
        let mut param = BytesMut::with_capacity(1);
        param.put_u8(address_types as u8);

        self.exec_command(
            ManagementCommand::StopDiscovery,
            controller,
            Some(param.to_bytes()),
            |_, param| Ok(FromPrimitive::from_u8(param.unwrap().get_u8()).unwrap()),
        )
            .await
    }

    /// This command can be used when the controller is not powered and
    ///	all settings will be programmed once powered.
    ///
    ///	The Source parameter selects the organization that assigned the
    ///	Vendor parameter:
    ///
    ///		0x0000	Disable Device ID
    ///		0x0001	Bluetooth SIG
    ///		0x0002	USB Implementer's Forum
    ///
    ///	The information is put into the EIR data. If the controller does
    ///	not support EIR or if SSP is disabled, this command will still
    ///	succeed. The information is stored for later use and will survive
    ///	toggling SSP on and off.
    pub async fn set_device_id(
        &mut self,
        controller: Controller,
        source: u16,
        vendor: u16,
        product: u16,
        version: u16,
    ) -> Result<()> {
        let mut param = BytesMut::with_capacity(8);
        param.put_u16_le(source);
        param.put_u16_le(vendor);
        param.put_u16_le(product);
        param.put_u16_le(version);

        self.exec_command(
            ManagementCommand::SetDeviceID,
            controller,
            Some(param.to_bytes()),
            |_, _| Ok(()),
        )
            .await
    }

    /// This command allows for setting the Low Energy scan parameters
    ///	used for connection establishment and passive scanning. It is
    ///	only supported on controllers with LE support.
    pub async fn set_scan_parameters(
        &mut self,
        controller: Controller,
        interval: u16,
        window: u16,
    ) -> Result<()> {
        let mut param = BytesMut::with_capacity(4);
        param.put_u16_le(interval);
        param.put_u16_le(window);

        self.exec_command(
            ManagementCommand::SetScanParameters,
            controller,
            Some(param.to_bytes()),
            |_, _| Ok(()),
        )
            .await
    }
}
