use super::*;

impl ManagementClient {
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
}