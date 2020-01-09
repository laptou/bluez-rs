use super::*;

fn address_callback(_: Controller, param: Option<Bytes>) -> Result<(Address, AddressType)> {
    let mut param = param.unwrap();
    Ok((
        Address::from_slice(param.split_to(6).as_ref()),
        FromPrimitive::from_u8(param.get_u8()).unwrap(),
    ))
}

impl ManagementClient {
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
            address_callback,
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
            address_callback,
        )
            .await
    }

    /// This command is used to remove a device from the list of blocked
    ///	devices (where it was added to using the Block Device command).
    ///
    ///	When the `address` parameter is `00:00:00:00:00:00`, then all
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
            address_callback,
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
            address_callback,
        )
            .await
    }

    ///	This command is used to retrieve a list of currently connected
    ///	devices.
    ///
    ///	For devices using resolvable random addresses with a known
    ///	identity resolving key, the `address` and `address_type` will
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
            address_callback,
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
  ///	The `address` and `address_type` of the return parameters will
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
            address_callback,
        )
            .await
    }

    ///	The `address` and `address_type` parameters should match what was
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
            address_callback,
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
            address_callback,
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
            address_callback,
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
            address_callback,
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
            address_callback,
        )
            .await
    }

    /// This command is used to remove data added using the Add Remote
    ///	Out Of Band Data command.
    ///
    ///	When the `address` parameter is `00:00:00:00:00:00`, then all
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
            address_callback,
        )
            .await
    }
}