use crate::AddressType;
use enumflags2::BitFlags;

use super::interact::{address_bytes, get_address};
use super::*;
use crate::util::BufExt;

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
pub async fn read_local_oob_data(
    socket: &mut ManagementStream,
    controller: Controller,
    event_tx: Option<mpsc::Sender<Response>>,
) -> Result<OutOfBandData> {
    let (_, param) = exec_command(
        socket,
        Command::ReadLocalOutOfBand,
        controller,
        None,
        event_tx,
    )
    .await?;

    let mut param = param.ok_or(Error::NoData)?;
    Ok(OutOfBandData {
        hash_192: param.get_array_u8(),
        randomizer_192: param.get_array_u8(),
        hash_256: if param.has_remaining() {
            Some(param.get_array_u8())
        } else {
            None
        },
        randomizer_256: if param.has_remaining() {
            Some(param.get_array_u8())
        } else {
            None
        },
    })
}

//  Right now, this command just returns the EIR data as a blob.
//  Maybe implement parsing later. See BT Core Spec sec 3.C.8, BT Core Spec Supplement Part A,
//  and https://www.bluetooth.com/specifications/assigned-numbers/generic-access-profile/
pub async fn read_local_oob_ext_data(
    socket: &mut ManagementStream,
    controller: Controller,
    address_types: BitFlags<AddressTypeFlag>,
    event_tx: Option<mpsc::Sender<Response>>,
) -> Result<(BitFlags<AddressTypeFlag>, Bytes)> {
    let (_, param) = exec_command(
        socket,
        Command::ReadLocalOutOfBandExtended,
        controller,
        Some(Bytes::copy_from_slice(&[address_types.bits() as u8])),
        event_tx,
    )
    .await?;

    let mut param = param.ok_or(Error::NoData)?;

    Ok((
        param.get_flags_u8(),
        // read eir data length param, then use that to split
        // should just end up splitting at the end but just to be safe
        {
            let eir_data_len = param.get_u16_le();
            param.split_to(eir_data_len as usize)
        },
    ))
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
    socket: &mut ManagementStream,
    controller: Controller,
    address: Address,
    address_type: AddressType,
    data: OutOfBandData,
    event_tx: Option<mpsc::Sender<Response>>,
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

    let (_, param) = exec_command(
        socket,
        Command::AddRemoteOutOfBand,
        controller,
        Some(param.freeze()),
        event_tx,
    )
    .await?;

    get_address(param)
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
    socket: &mut ManagementStream,
    controller: Controller,
    address: Address,
    address_type: AddressType,
    event_tx: Option<mpsc::Sender<Response>>,
) -> Result<(Address, AddressType)> {
    let (_, param) = exec_command(
        socket,
        Command::RemoveRemoteOutOfBand,
        controller,
        Some(address_bytes(address, address_type)),
        event_tx,
    )
    .await?;

    get_address(param)
}

#[derive(Debug)]
pub struct OutOfBandData {
    pub hash_192: [u8; 16],
    pub randomizer_192: [u8; 16],
    pub hash_256: Option<[u8; 16]>,
    pub randomizer_256: Option<[u8; 16]>,
}
