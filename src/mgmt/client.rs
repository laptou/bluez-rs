use std::convert::TryInto;
use std::ffi::CString;

use bytes::*;
use num_traits::FromPrimitive;

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
    // use some consts for common callback patterns
    fn settings_callback(_: Controller, param: Option<Bytes>) -> Result<ControllerSettings> {
        let mut param = param.unwrap();
        Ok(ControllerSettings::from_bits_truncate(param.get_u32_le()))
    }

    fn address_callback(_: Controller, param: Option<Bytes>) -> Result<(Address, AddressType)> {
        let mut param = param.unwrap();
        Ok((
            Address::from_slice(param.split_to(6).as_ref()),
            FromPrimitive::from_u8(param.get_u8()).unwrap(),
        ))
    }

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

        self.exec_command(
            ManagementCommand::SetPowered,
            controller,
            Some(param.to_bytes()),
            Self::settings_callback,
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

        self.exec_command(
            ManagementCommand::SetDiscoverable,
            controller,
            Some(param.to_bytes()),
            Self::settings_callback,
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

        self.exec_command(
            ManagementCommand::SetConnectable,
            controller,
            Some(param.to_bytes()),
            Self::settings_callback,
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

        self.exec_command(
            ManagementCommand::SetFastConnectable,
            controller,
            Some(param.to_bytes()),
            Self::settings_callback,
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

        self.exec_command(
            ManagementCommand::SetPairable,
            controller,
            Some(param.to_bytes()),
            Self::settings_callback,
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

        self.exec_command(
            ManagementCommand::SetLinkSecurity,
            controller,
            Some(param.to_bytes()),
            Self::settings_callback,
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

        self.exec_command(
            ManagementCommand::SetSecureSimplePairing,
            controller,
            Some(param.to_bytes()),
            Self::settings_callback,
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

        self.exec_command(
            ManagementCommand::SetHighSpeed,
            controller,
            Some(param.to_bytes()),
            Self::settings_callback,
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

        self.exec_command(
            ManagementCommand::SetLowEnergy,
            controller,
            Some(param.to_bytes()),
            Self::settings_callback,
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
            Self::address_callback,
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
                        FromPrimitive::from_u8(param.get_u8()).unwrap(),
                    ));
                }

                Ok(connections)
            },
        )
            .await
    }

    ///	This command is used to respond to a PIN Code request event.
    /// Pin code can be at most 16 bytes. Passing None will send a
    /// negative PIN code response.
    ///	This command can only be used when the controller is powered.
    pub async fn pin_code_reply(
        &mut self,
        controller: Controller,
        address: Address,
        address_type: AddressType,
        pin_code: Option<Vec<u8>>,
    ) -> Result<(Address, AddressType)> {
        let mut param;
        let opcode;

        if let Some(pin_code) = pin_code {
            opcode = ManagementCommand::PinCodeReply;
            param = BytesMut::with_capacity(24);
            param.put_slice(address.as_ref());
            param.put_u8(address_type as u8);
            param.put_u8(pin_code.len() as u8);
            param.put_slice(&pin_code[..]);
            param.resize(24, 0);
        } else {
            opcode = ManagementCommand::PinCodeNegativeReply;
            param = BytesMut::with_capacity(7);
            param.put_slice(address.as_ref());
            param.put_u8(address_type as u8);
        }

        self.exec_command(
            opcode,
            controller,
            Some(param.to_bytes()),
            Self::address_callback,
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

    ///	This command is used to trigger pairing with a remote device.
    ///	The IO_Capability command parameter is used to temporarily (for
    ///	this pairing event only) override the global IO Capability (set
    ///	using the Set IO Capability command).
    ///
    ///	Passing a value 4 (KeyboardDisplay) will cause the kernel to
    ///	convert it to 1 (DisplayYesNo) in the case of a BR/EDR
    ///	connection (as KeyboardDisplay is specific to SMP).
    ///
    ///	The Address and Address_Type of the return parameters will
    ///	return the identity address if known. In case of resolvable
    ///	random address given as command parameters and the remote
    ///	provides an identity resolving key, the return parameters
    ///	will provide the resolved address.
    ///
    ///	To allow tracking of which resolvable random address changed
    ///	into which identity address, the New Identity Resolving Key
    ///	event will be sent before receiving Command Complete event
    ///	for this command.
    ///
    ///	This command can only be used when the controller is powered.
    pub async fn pair_device(
        &mut self,
        controller: Controller,
        address: Address,
        address_type: AddressType,
        io_capability: IoCapability,
    ) -> Result<(Address, AddressType)> {
        let mut param = BytesMut::with_capacity(8);
        param.put_slice(address.as_ref());
        param.put_u8(address_type as u8);
        param.put_u8(io_capability as u8);

        self.exec_command(
            ManagementCommand::PairDevice,
            controller,
            Some(param.to_bytes()),
            Self::address_callback,
        )
            .await
    }

    ///	The Address and Address_Type parameters should match what was
    ///	given to a preceding Pair Device command.
    ///
    ///	This command can only be used when the controller is powered.
    pub async fn cancel_pair_device(
        &mut self,
        controller: Controller,
        address: Address,
        address_type: AddressType,
    ) -> Result<(Address, AddressType)> {
        let mut param = BytesMut::with_capacity(7);
        param.put_slice(address.as_ref());
        param.put_u8(address_type as u8);

        self.exec_command(
            ManagementCommand::CancelPairDevice,
            controller,
            Some(param.to_bytes()),
            Self::address_callback,
        )
            .await
    }

    ///	Removes all keys associated with the remote device.
    ///
    ///	The disconnect parameter tells the kernel whether to forcefully
    ///	disconnect any existing connections to the device. It should in
    ///	practice always be true except for some special GAP qualification
    ///	test-cases where a key removal without disconnecting is needed.
    ///
    ///	When unpairing a device its link key, long term key and if
    ///	provided identity resolving key will be purged.
    ///
    ///	For devices using resolvable random addresses where the identity
    ///	resolving key was available, after this command they will now no
    ///	longer be resolved. The device will essentially become private
    ///	again.
    ///
    ///	This command can only be used when the controller is powered.
    pub async fn unpair_device(
        &mut self,
        controller: Controller,
        address: Address,
        address_type: AddressType,
        disconnect: bool,
    ) -> Result<(Address, AddressType)> {
        let mut param = BytesMut::with_capacity(8);
        param.put_slice(address.as_ref());
        param.put_u8(address_type as u8);
        param.put_u8(disconnect as u8);

        self.exec_command(
            ManagementCommand::UnpairDevice,
            controller,
            Some(param.to_bytes()),
            Self::address_callback,
        )
            .await
    }

    ///	This command is used to respond to a User Confirmation Request
    ///	event. This command can only be used when the controller is powered.
    pub async fn user_confirmation_reply(
        &mut self,
        controller: Controller,
        address: Address,
        address_type: AddressType,
        reply: bool,
    ) -> Result<(Address, AddressType)> {
        let mut param = BytesMut::with_capacity(7);
        param.put_slice(address.as_ref());
        param.put_u8(address_type as u8);

        self.exec_command(
            if reply {
                ManagementCommand::UserConfirmationReply
            } else {
                ManagementCommand::UserConfirmationNegativeReply
            },
            controller,
            Some(param.to_bytes()),
            Self::address_callback,
        )
            .await
    }

    ///	This command is used to respond to a User Passkey Request
    ///	event. Passing None for passkey will send a negative response.
    /// This command can only be used when the controller is powered.
    pub async fn user_passkey_reply(
        &mut self,
        controller: Controller,
        address: Address,
        address_type: AddressType,
        passkey: Option<u32>,
    ) -> Result<(Address, AddressType)> {
        let opcode;
        let mut param;

        if let Some(passkey) = passkey {
            opcode = ManagementCommand::UserPasskeyReply;
            param = BytesMut::with_capacity(11);
            param.put_slice(address.as_ref());
            param.put_u8(address_type as u8);
            param.put_u32_le(passkey);
        } else {
            opcode = ManagementCommand::UserPasskeyNegativeReply;
            param = BytesMut::with_capacity(7);
            param.put_slice(address.as_ref());
            param.put_u8(address_type as u8);
        }

        self.exec_command(
            opcode,
            controller,
            Some(param.to_bytes()),
            Self::address_callback,
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

    ///	This command is used to provide Out of Band data for a remote
    ///	device.
    ///
    ///	Provided Out Of Band data is persistent over power down/up toggles.
    ///
    ///	This command also accept optional P-256 versions of hash and
    ///	randomizer. If they are not provided, then they are set to
    ///	zero value.
    ///
    ///	The P-256 versions of both can also be provided when the
    ///	support for Secure Connections is not enabled. However in
    ///	that case they will never be used.
    ///
    ///	To only provide the P-256 versions of hash and randomizer,
    ///	it is valid to leave both P-192 fields as zero values. If
    ///	Secure Connections is disabled, then of course this is the
    ///	same as not providing any data at all.
    ///
    ///	When providing data for remote LE devices, then the Hash_192 and
    ///	and Randomizer_192 fields are not used and shell be set to zero.
    ///
    ///	The Hash_256 and Randomizer_256 fields can be used for LE secure
    ///	connections Out Of Band data. If only LE secure connections data
    ///	is provided the Hash_P192 and Randomizer_P192 fields can be set
    ///	to zero. Currently there is no support for providing the Security
    ///	Manager TK Value for LE legacy pairing.
    ///
    ///	If Secure Connections Only mode has been enabled, then providing
    ///	Hash_P192 and Randomizer_P192 is not allowed. They are required
    ///	to be set to zero values.
    ///
    ///	This command can be used when the controller is not powered and
    ///	all settings will be programmed once powered.
    pub async fn add_remote_oob_data(
        &mut self,
        controller: Controller,
        address: Address,
        address_type: AddressType,
        data: OutOfBandData,
    ) -> Result<(Address, AddressType)> {
        let mut param = BytesMut::with_capacity(39);
        param.put_slice(address.as_ref());
        param.put_u8(address_type as u8);
        param.put_slice(&data.hash_192[..]);
        param.put_slice(&data.randomizer_192[..]);

        if let Some(hash_256) = data.hash_256 {
            param.put_slice(&hash_256[..]);
        }
        if let Some(randomizer_256) = data.randomizer_256 {
            param.put_slice(&randomizer_256[..]);
        }

        self.exec_command(
            ManagementCommand::AddRemoteOutOfBand,
            controller,
            Some(param.to_bytes()),
            Self::address_callback,
        )
            .await
    }

    /// This command is used to remove data added using the Add Remote
    ///	Out Of Band Data command.
    ///
    ///	When the Address parameter is 00:00:00:00:00:00, then all
    ///	previously added data will be removed.
    ///
    ///	This command can be used when the controller is not powered and
    ///	all settings will be programmed once powered.
    pub async fn remove_remote_oob_data(
        &mut self,
        controller: Controller,
        address: Address,
        address_type: AddressType,
    ) -> Result<(Address, AddressType)> {
        let mut param = BytesMut::with_capacity(39);
        param.put_slice(address.as_ref());
        param.put_u8(address_type as u8);

        self.exec_command(
            ManagementCommand::RemoveRemoteOutOfBand,
            controller,
            Some(param.to_bytes()),
            Self::address_callback,
        )
            .await
    }

    ///	This command is used to start the process of discovering remote
    ///	devices. A Device Found event will be sent for each discovered
    ///	device.
    ///
    ///	Possible values for the Address_Type parameter are a bit-wise or
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

    ///	This command is only valid during device discovery and is
    ///	expected for each Device Found event with the Confirm Name
    ///	flag set.
    ///
    ///	The name_known parameter should be set to true if user space
    ///	knows the name for the device and false if it doesn't. If set to
    ///	false the kernel will perform a name resolving procedure for the
    ///	device in question.
    ///
    ///	This command can only be used when the controller is powered.
    pub async fn confirm_name(
        &mut self,
        controller: Controller,
        address: Address,
        address_type: AddressType,
        name_known: bool,
    ) -> Result<(Address, AddressType)> {
        let mut param = BytesMut::with_capacity(8);
        param.put_slice(address.as_ref());
        param.put_u8(address_type as u8);
        param.put_u8(name_known as u8);

        self.exec_command(
            ManagementCommand::ConfirmName,
            controller,
            Some(param.to_bytes()),
            Self::address_callback,
        )
            .await
    }

    /// This command is used to add a device to the list of devices
    ///	which should be blocked from being connected to the local
    ///	controller.
    ///
    ///	For Low Energy devices, the blocking of a device takes precedence
    ///	over auto-connection actions provided by Add Device. Blocked
    ///	devices will not be auto-connected or even reported when found
    ///	during background scanning. If the controller is connectable
    ///	direct advertising from blocked devices will also be ignored.
    ///
    ///	Connections created from advertising of the controller will
    ///	be dropped if the device is blocked.
    ///
    ///	This command can be used when the controller is not powered.
    pub async fn block_device(
        &mut self,
        controller: Controller,
        address: Address,
        address_type: AddressType,
    ) -> Result<(Address, AddressType)> {
        let mut param = BytesMut::with_capacity(7);
        param.put_slice(address.as_ref());
        param.put_u8(address_type as u8);

        self.exec_command(
            ManagementCommand::BlockDevice,
            controller,
            Some(param.to_bytes()),
            Self::address_callback,
        )
            .await
    }

    /// This command is used to remove a device from the list of blocked
    ///	devices (where it was added to using the Block Device command).
    ///
    ///	When the Address parameter is 00:00:00:00:00:00, then all
    ///	previously blocked devices will be unblocked.
    ///
    ///	This command can be used when the controller is not powered.
    pub async fn unblock_device(
        &mut self,
        controller: Controller,
        address: Address,
        address_type: AddressType,
    ) -> Result<(Address, AddressType)> {
        let mut param = BytesMut::with_capacity(7);
        param.put_slice(address.as_ref());
        param.put_u8(address_type as u8);

        self.exec_command(
            ManagementCommand::UnblockDevice,
            controller,
            Some(param.to_bytes()),
            Self::address_callback,
        )
            .await
    }

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

    pub async fn set_advertising(
        &mut self,
        controller: Controller,
        advertising: bool,
    ) -> Result<ControllerSettings> {
        let mut param = BytesMut::with_capacity(1);
        param.put_u8(advertising as u8);

        self.exec_command(
            ManagementCommand::SetAdvertising,
            controller,
            Some(param.to_bytes()),
            Self::settings_callback,
        )
            .await
    }
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, FromPrimitive)]
pub enum DiscoveryAddressTypes {
    /// BR/EDR
    BrEdr = 1,
    /// LE (public & random)
    LE = 6,
    /// BR/EDR/LE (interleaved discovery)
    BrEdrLE = 7,
}

#[derive(Debug)]
pub struct LinkKey {
    address: Address,
    address_type: AddressType,
    key_type: LinkKeyType,
    value: [u8; 16],
    pin_length: u8,
}

#[derive(Copy, Clone, Debug)]
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

#[derive(Debug)]
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

#[derive(Copy, Clone, Debug)]
#[repr(u8)]
pub enum LongTermKeyType {
    Unauthenticated = 0x00,
    Authenticated = 0x01,
}

#[derive(Copy, Clone, Debug)]
#[repr(u8)]
pub enum IoCapability {
    DisplayOnly = 1,
    DisplayYesNo,
    KeyboardOnly,
    NoInputNoOutput,
    KeyboardDisplay,
}

#[derive(Debug)]
pub struct OutOfBandData {
    hash_192: [u8; 16],
    randomizer_192: [u8; 16],
    hash_256: Option<[u8; 16]>,
    randomizer_256: Option<[u8; 16]>,
}
