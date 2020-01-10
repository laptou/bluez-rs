use super::*;

impl ManagementClient {
    /// This command is used to feed the kernel with currently known
    ///	link keys. The command does not need to be called again upon the
    ///	receipt of New Link Key events since the kernel updates its list
    ///	automatically.
    ///
    ///	The debug parameter is used to tell the kernel whether to
    ///	accept the usage of debug keys or not. The allowed values for
    ///	this parameter are `0x00` and `0x01`. All other values will return
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
        let mut param = BytesMut::with_capacity(2 + keys.len() * 32);
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

    ///	This command is used to feed the kernel with currently known
    ///	identity resolving keys. The command does not need to be called
    ///	again upon the receipt of New Identity Resolving Key events
    ///	since the kernel updates its list automatically.
    ///
    ///	The provided `address` and `address_type` are the identity of
    ///	a device. So either its public address or static random address.
    ///
    ///	Unresolvable random addresses and resolvable random addresses are
    ///	not valid and will be rejected.
    ///
    ///	This command can be used when the controller is not powered.
    pub async fn load_identity_resolving_keys(
        &mut self,
        controller: Controller,
        keys: Vec<IdentityResolvingKey>,
    ) -> Result<()> {
        let mut param = BytesMut::with_capacity(2 + keys.len() * 23);
        param.put_u16_le(keys.len() as u16);

        for key in keys {
            param.put_slice(key.address.as_ref());
            param.put_u8(key.address_type as u8);
            param.put_slice(&key.value[..]);
        }

        self.exec_command(
            ManagementCommand::LoadIdentityResolvingKeys,
            controller,
            Some(param.to_bytes()),
            |_, _| Ok(()),
        )
        .await
    }

    ///	This command is used to load connection parameters from several
    ///	devices into kernel. Currently this is only supported on controllers
    ///	with Low Energy support.
    ///
    ///	The provided Address and Address_Type are the identity of
    ///	a device. So either its public address or static random address.
    ///
    ///	The `min_connection_interval`, `max_connection_interval`,
    ///	`connection_latency` and `supervision_timeout` parameters should
    ///	be configured as described in Core 4.1 spec, Vol 2, 7.8.12.
    ///
    ///	This command can be used when the controller is not powered.
    pub async fn load_connection_parameters(
        &mut self,
        controller: Controller,
        connection_params: Vec<ConnectionParameters>,
    ) -> Result<()> {
        let mut param = BytesMut::with_capacity(2 + connection_params.len() * 15);
        param.put_u16_le(connection_params.len() as u16);

        for cxn_param in connection_params {
            param.put_slice(cxn_param.address.as_ref());
            param.put_u8(cxn_param.address_type as u8);
            param.put_u16_le(cxn_param.min_connection_interval);
            param.put_u16_le(cxn_param.max_connection_interval);
            param.put_u16_le(cxn_param.connection_latency);
            param.put_u16_le(cxn_param.supervision_timeout);
        }

        self.exec_command(
            ManagementCommand::LoadConnectionParameters,
            controller,
            Some(param.to_bytes()),
            |_, _| Ok(()),
        )
        .await
    }

    /// This command is used to feed the kernel a list of keys that
    ///	are known to be vulnerable.
    ///
    ///	If the pairing procedure produces any of these keys, they will be
    ///	silently dropped and any attempt to enable encryption rejected.
    ///
    /// This command can be used when the controller is not powered.
    pub async fn load_blocked_keys(
        &mut self,
        controller: Controller,
        keys: Vec<BlockedKey>,
    ) -> Result<()> {
        let mut param = BytesMut::with_capacity(2 + keys.len() * 17);
        param.put_u16_le(keys.len() as u16);

        for key in keys {
            param.put_u8(key.key_type as u8);
            param.put_slice(&key.value[..]);
        }

        self.exec_command(
            ManagementCommand::LoadBlockedKeys,
            controller,
            Some(param.to_bytes()),
            |_, _| Ok(()),
        )
        .await
    }
}
