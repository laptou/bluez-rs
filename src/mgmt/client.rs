use std::convert::TryInto;
use std::ffi::CString;

use bytes::*;
use enumflags2::_internal::core::ops::Add;

use crate::Address;
use crate::mgmt::{ManagementError, Result};
use crate::mgmt::interface::{
    class, Discoverability, ManagementCommand, ManagementCommandStatus, ManagementRequest,
};
use crate::mgmt::interface::class::{DeviceClass, ServiceClasses};
use crate::mgmt::interface::controller::{Controller, ControllerInfo, ControllerSettings};
use crate::mgmt::interface::event::{AddressType, ManagementEvent, ManagementVersion};
use crate::mgmt::socket::ManagementSocket;

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

    #[inline]
    fn exec_settings_command(
        &mut self,
        opcode: ManagementCommand,
        controller: Controller,
        param: Option<Bytes>,
    ) -> impl futures::Future<Output=Result<ControllerSettings>> + '_ {
        return self.exec_command(opcode, controller, param, |_, param| {
            let mut param = param.unwrap();
            Ok(ControllerSettings::from_bits_truncate(param.get_u32_le()))
        });
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
    ///	The Address parameter describes the controllers public address
    ///	and it can be expected that it is set. However in case of single
    ///	mode Low Energy only controllers it can be 00:00:00:00:00:00. To
    ///	power on the controller in this case, it is required to configure
    ///	a static address using Set Static Address command first.
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

    /// This command is used to power on or off a controller.
    ///
    ///	If discoverable setting is activated with a timeout, then
    ///	switching the controller off will expire this timeout and
    ///	disable discoverable.
    ///
    ///	Settings programmed via Set Advertising and Add/Remove
    ///	Advertising while the controller was powered off will be activated
    ///	when powering the controller on.
    ///
    ///	Switching the controller off will permanently cancel and remove
    ///	all advertising instances with a timeout set, i.e. time limited
    ///	advertising instances are not being remembered across power cycles.
    ///	Advertising Removed events will be issued accordingly.
    pub async fn set_powered(
        &mut self,
        controller: Controller,
        powered: bool,
    ) -> Result<ControllerSettings> {
        let mut param = BytesMut::with_capacity(1);
        param.put_u8(powered as u8);

        self.exec_settings_command(
            ManagementCommand::SetPowered,
            controller,
            Some(param.to_bytes()),
        )
            .await
    }

    /// This command is used to set the discoverable property of a
    ///	controller.
    ///
    ///	Timeout is the time in seconds and is only meaningful when
    ///	Discoverable is set to General or Limited. Providing a timeout
    ///	with None returns Invalid Parameters. For Limited, the timeout
    ///	is required.
    ///
    ///	This command is only available for BR/EDR capable controllers
    ///	(e.g. not for single-mode LE ones). It will return Not Supported
    ///	otherwise.
    ///
    ///	This command can be used when the controller is not powered and
    ///	all settings will be programmed once powered, however using a timeout
    /// when the controller is not powered will	return Not Powered error.
    ///
    ///	When switching discoverable on and the connectable setting is
    ///	off it will return Rejected error.
    pub async fn set_discoverable(
        &mut self,
        controller: Controller,
        discoverability: Discoverability,
        timeout: Option<u16>,
    ) -> Result<ControllerSettings> {
        let mut param = BytesMut::with_capacity(3);
        param.put_u8(discoverability as u8);
        if let Some(timeout) = timeout {
            param.put_u16_le(timeout);
        }

        self.exec_settings_command(
            ManagementCommand::SetDiscoverable,
            controller,
            Some(param.to_bytes()),
        )
            .await
    }

    /// This command is used to set the connectable property of a
    ///	controller.
    ///
    ///	This command is available for BR/EDR, LE-only and also dual
    ///	mode controllers. For BR/EDR is changes the page scan setting
    ///	and for LE controllers it changes the advertising type. For
    ///	dual mode controllers it affects both settings.
    ///
    ///	For LE capable controllers the connectable setting takes effect
    ///	when advertising is enabled (peripheral) or when directed
    ///	advertising events are received (central).
    ///
    ///	This command can be used when the controller is not powered and
    ///	all settings will be programmed once powered.
    ///
    ///	When switching connectable off, it will also switch off the
    ///	discoverable setting. Switching connectable back on will not
    ///	restore a previous discoverable. It will stay off and needs
    ///	to be manually switched back on.
    ///
    ///	When switching connectable off, it will expire a discoverable
    ///	setting with a timeout.
    ///
    ///	This setting does not affect known devices from Add Device
    ///	command. These devices are always allowed to connect.
    pub async fn set_connectable(
        &mut self,
        controller: Controller,
        connectable: bool,
    ) -> Result<ControllerSettings> {
        let mut param = BytesMut::with_capacity(1);
        param.put_u8(connectable as u8);

        self.exec_settings_command(
            ManagementCommand::SetConnectable,
            controller,
            Some(param.to_bytes()),
        )
            .await
    }

    /// This command is used to set the controller into a connectable
    ///	state where the page scan parameters have been set in a way to
    ///	favor faster connect times with the expense of higher power
    ///	consumption.
    ///
    ///	This command is only available for BR/EDR capable controllers
    ///	(e.g. not for single-mode LE ones). It will return Not Supported
    ///	otherwise.
    ///
    ///	This command can be used when the controller is not powered and
    ///	all settings will be programmed once powered.
    ///
    ///	The setting will be remembered during power down/up toggles.
    pub async fn set_fast_connectable(
        &mut self,
        controller: Controller,
        fast_connectable: bool,
    ) -> Result<ControllerSettings> {
        let mut param = BytesMut::with_capacity(1);
        param.put_u8(fast_connectable as u8);

        self.exec_settings_command(
            ManagementCommand::SetFastConnectable,
            controller,
            Some(param.to_bytes()),
        )
            .await
    }

    /// This command is used to set the bondable (pairable) property of an
    ///	controller.
    ///
    ///	This command can be used when the controller is not powered and
    ///	all settings will be programmed once powered.
    ///
    ///	Turning bondable on will not automatically switch the controller
    ///	into connectable mode. That needs to be done separately.
    ///
    ///	The setting will be remembered during power down/up toggles.
    pub async fn set_bondable(
        &mut self,
        controller: Controller,
        bondable: bool,
    ) -> Result<ControllerSettings> {
        let mut param = BytesMut::with_capacity(1);
        param.put_u8(bondable as u8);

        self.exec_settings_command(
            ManagementCommand::SetPairable,
            controller,
            Some(param.to_bytes()),
        )
            .await
    }

    ///	This command is used to either enable or disable link level
    ///	security for an controller (also known as Security Mode 3).
    ///
    ///	This command is only available for BR/EDR capable controllers
    ///	(e.g. not for single-mode LE ones). It will return Not Supported
    ///	otherwise.
    ///
    ///	This command can be used when the controller is not powered and
    ///	all settings will be programmed once powered.
    pub async fn set_link_security(
        &mut self,
        controller: Controller,
        link_security: bool,
    ) -> Result<ControllerSettings> {
        let mut param = BytesMut::with_capacity(1);
        param.put_u8(link_security as u8);

        self.exec_settings_command(
            ManagementCommand::SetLinkSecurity,
            controller,
            Some(param.to_bytes()),
        )
            .await
    }

    ///	This command is used to enable/disable Secure Simple Pairing
    ///	support for a controller.
    ///
    ///	This command is only available for BR/EDR capable controllers
    ///	supporting the core specification version 2.1 or greater
    ///	(e.g. not for single-mode LE controllers or pre-2.1 ones).
    ///
    ///	This command can be used when the controller is not powered and
    ///	all settings will be programmed once powered.
    ///
    ///	In case the controller does not support Secure Simple Pairing,
    ///	the command will fail regardless with Not Supported error.
    pub async fn set_ssp(
        &mut self,
        controller: Controller,
        ssp: bool,
    ) -> Result<ControllerSettings> {
        let mut param = BytesMut::with_capacity(1);
        param.put_u8(ssp as u8);

        self.exec_settings_command(
            ManagementCommand::SetSecureSimplePairing,
            controller,
            Some(param.to_bytes()),
        )
            .await
    }

    ///	This command is used to enable/disable Bluetooth High Speed
    ///	support for a controller.
    ///
    ///	This command is only available for BR/EDR capable controllers
    ///	(e.g. not for single-mode LE ones).
    ///
    ///	This command can be used when the controller is not powered and
    ///	all settings will be programmed once powered.
    ///
    ///	To enable High Speed support, it is required that Secure Simple
    ///	Pairing support is enabled first. High Speed support is not
    ///	possible for connections without Secure Simple Pairing.
    ///
    ///	When switching Secure Simple Pairing off, the support for High
    ///	Speed will be switched off as well. Switching Secure Simple
    ///	Pairing back on, will not re-enable High Speed support. That
    ///	needs to be done manually.
    pub async fn set_high_speed(
        &mut self,
        controller: Controller,
        high_speed: bool,
    ) -> Result<ControllerSettings> {
        let mut param = BytesMut::with_capacity(1);
        param.put_u8(high_speed as u8);

        self.exec_settings_command(
            ManagementCommand::SetHighSpeed,
            controller,
            Some(param.to_bytes()),
        )
            .await
    }

    /// This command is used to enable/disable Low Energy support for a
    ///	controller.
    ///
    ///	This command is only available for LE capable controllers and
    ///	will yield in a Not Supported error otherwise.
    ///
    ///	This command can be used when the controller is not powered and
    ///	all settings will be programmed once powered.
    ///
    ///	In case the kernel subsystem does not support Low Energy or the
    ///	controller does not either, the command will fail regardless.
    ///
    ///	Disabling LE support will permanently disable and remove all
    ///	advertising instances configured with the Add Advertising
    ///	command. Advertising Removed events will be issued accordingly.
    pub async fn set_le(&mut self, controller: Controller, le: bool) -> Result<ControllerSettings> {
        let mut param = BytesMut::with_capacity(1);
        param.put_u8(le as u8);

        self.exec_settings_command(
            ManagementCommand::SetLowEnergy,
            controller,
            Some(param.to_bytes()),
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

    ///	This command is used to force the disconnection of a currently
    ///	connected device.
    ///
    ///	This command can only be used when the controller is powered.
    pub async fn disconnect(
        &mut self,
        controller: Controller,
        address: Address,
        address_type: AddressType,
    ) -> Result<(Address, AddressType)> {
        let mut param = BytesMut::with_capacity(7);
        param.put_slice(address.as_ref());
        param.put_u8(address_type as u8);

        self.exec_command(
            ManagementCommand::Disconnect,
            controller,
            Some(param.to_bytes()),
            |_, param| {
                let mut param = param.unwrap();
                Ok((
                    Address::from_slice(param.split_to(6).as_ref()),
                    param.get_u8() as AddressType,
                ))
            },
        )
            .await
    }

    ///	This command is used to retrieve a list of currently connected
    ///	devices.
    ///
    ///	For devices using resolvable random addresses with a known
    ///	identity resolving key, the Address and Address_Type will
    ///	contain the identity information.
    ///
    ///	This command can only be used when the controller is powered.
    pub async fn get_connections(
        &mut self,
        controller: Controller,
    ) -> Result<Vec<(Address, AddressType)>> {
        self.exec_command(
            ManagementCommand::GetConnections,
            controller,
            None,
            |_, param| {
                let mut param = param.unwrap();
                let count = param.get_u16_le() as usize;
                let mut connections = Vec::with_capacity(count);

                for _ in 0..count {
                    connections.push((
                        Address::from_slice(param.split_to(6).as_ref()),
                        param.get_u8() as AddressType,
                    ));
                }

                Ok(connections)
            },
        )
            .await
    }

    ///	This command is used to respond to a PIN Code request event.
    /// Pin code can be at most 16 bytes.
    ///	This command can only be used when the controller is powered.
    pub async fn pin_code_reply(
        &mut self,
        controller: Controller,
        address: Address,
        address_type: AddressType,
        pin_code: Vec<u8>,
    ) -> Result<(Address, AddressType)> {
        let mut param = BytesMut::with_capacity(24);
        param.put_slice(address.as_ref());
        param.put_u8(address_type as u8);
        param.put_u8(pin_code.len() as u8);
        param.put_slice(&pin_code[..]);
        param.resize(24, 0);

        self.exec_command(
            ManagementCommand::PinCodeReply,
            controller,
            Some(param.to_bytes()),
            |_, param| {
                let mut param = param.unwrap();
                Ok((
                    Address::from_slice(param.split_to(6).as_ref()),
                    param.get_u8() as AddressType,
                ))
            },
        )
            .await
    }
}

pub struct LinkKey {
    address: Address,
    address_type: AddressType,
    key_type: LinkKeyType,
    value: [u8; 16],
    pin_length: u8,
}

#[repr(u8)]
pub enum LinkKeyType {
    Combination = 0x00,
    LocalUnit = 0x01,
    RemoteUnit = 0x02,
    DebugCombination = 0x03,
    UnauthenticatedCombinationP192 = 0x04,
    AuthenticatedCombinationP192 = 0x05,
    ChangedCombination = 0x06,
    UnauthenticatedCombinationP256 = 0x07,
    AuthenticatedCombinationP256 = 0x08,
}

pub struct LongTermKey {
    address: Address,
    address_type: AddressType,
    key_type: LongTermKeyType,
    master: u8,
    encryption_size: u8,
    encryption_diversifier: u16,
    random_number: u64,
    value: [u8; 16],
}

#[repr(u8)]
pub enum LongTermKeyType {
    Unauthenticated = 0x00,
    Authenticated = 0x01,
}
