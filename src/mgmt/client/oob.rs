use super::*;

impl ManagementClient {
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
}
