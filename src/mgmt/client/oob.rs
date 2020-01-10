use enumflags2::BitFlags;

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

    //  Right now, this command just returns the EIR data as a blob.
    //  Maybe implement parsing later. See BT Core Spec sec 3.C.8, BT Core Spec Supplement Part A,
    //  and https://www.bluetooth.com/specifications/assigned-numbers/generic-access-profile/
    pub async fn read_local_oob_ext_data(
        &mut self,
        controller: Controller,
        address_types: BitFlags<AddressTypeFlag>,
    ) -> Result<(BitFlags<AddressTypeFlag>, Bytes)> {
        self.exec_command(
            ManagementCommand::ReadLocalOutOfBandExtended,
            controller,
            Some(BytesMut::from([address_types.bits() as u8].as_ref() as &[u8]).to_bytes()),
            |_, param| {
                let mut param = param.unwrap();
                let address_types = BitFlags::from_bits_truncate(param.get_u8());
                let eir_data_len = param.get_u16_le();
                Ok((
                    address_types,
                    // read eir data length param, then use that to split
                    // should just end up splitting at the end but just to be safe
                    param.split_to(eir_data_len as usize),
                ))
            },
        )
        .await
    }
}
