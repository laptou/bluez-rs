use enumflags2::BitFlags;

use super::*;
use crate::util::BufExt;

///	This command is used to start the process of discovering remote
///	devices. A Device Found event will be sent for each discovered
///	device.
///
///	Possible values for the `address_type` parameter are a bit-wise or
///	of the following bits:
///
///	0	BR/EDR
///	1	LE Public
///	2	LE Random
///
///	By combining these e.g. the following values are possible:
///
///	1	BR/EDR
///	6	LE (public & random)
///	7	BR/EDR/LE (interleaved discovery)
///
///	This command can only be used when the controller is powered.
pub async fn start_discovery(
    socket: &mut ManagementStream,
    controller: Controller,
    address_types: BitFlags<AddressTypeFlag>,
    event_tx: Option<mpsc::Sender<Response>>,
) -> Result<BitFlags<AddressTypeFlag>> {
    let mut param = BytesMut::with_capacity(1);
    param.put_u8(address_types.bits());

    let (_, param) = exec_command(
        socket,
        Command::StartDiscovery,
        controller,
        Some(param.freeze()),
        event_tx,
    )
    .await?;

    Ok(param.ok_or(Error::NoData)?.get_flags_u8())
}

/// This command is used to stop the discovery process started using
///	the Start Discovery command.
///
///	This command can only be used when the controller is powered.
pub async fn stop_discovery(
    socket: &mut ManagementStream,
    controller: Controller,
    address_types: BitFlags<AddressTypeFlag>,
    event_tx: Option<mpsc::Sender<Response>>,
) -> Result<BitFlags<AddressTypeFlag>> {
    let mut param = BytesMut::with_capacity(1);
    param.put_u8(address_types.bits());

    let (_, param) = exec_command(
        socket,
        Command::StopDiscovery,
        controller,
        Some(param.freeze()),
        event_tx,
    )
    .await?;

    Ok(param.ok_or(Error::NoData)?.get_flags_u8())
}

///	This command is used to start the process of discovering remote
///	devices with a specific UUID. A Device Found event will be sent
///	for each discovered device.
///
///  The service discovery uses active scanning for Low Energy scanning
///	and will search for UUID in both advertising data and scan response
///	data.
///
///	Found devices that have a RSSI value smaller than `rssi_threshold`
///	are not reported via DeviceFound event. Setting a value of 127
///	will cause all devices to be reported.
///
///	The list of UUIDs identifies a logical OR. Only one of the UUIDs
///	have to match to cause a DeviceFound event. Providing an empty
///	list of UUIDs means that DeviceFound events are send out for all devices above the RSSI_Threshold.
///
///	In case `rssi_threshold` is set to 127 and `uuids` is empty, then
///	this command behaves exactly the same as Start Discovery.
///
///	When the discovery procedure starts the Discovery event will
///	notify this similar to Start Discovery.
///
///	This command can only be used when the controller is powered.
pub async fn start_service_discovery(
    socket: &mut ManagementStream,
    controller: Controller,
    address_types: BitFlags<AddressTypeFlag>,
    rssi_threshold: i8,
    uuids: Vec<[u8; 16]>,
    event_tx: Option<mpsc::Sender<Response>>,
) -> Result<BitFlags<AddressTypeFlag>> {
    let mut param = BytesMut::with_capacity(4 + 16 * uuids.len());
    param.put_u8(address_types.bits());
    param.put_i8(rssi_threshold);
    param.put_u16_le(uuids.len() as u16);

    for uuid in uuids {
        param.put_slice(&uuid[..]);
    }

    let (_, param) = exec_command(
        socket,
        Command::StartServiceDiscovery,
        controller,
        Some(param.freeze()),
        event_tx,
    )
    .await?;

    Ok(param.ok_or(Error::NoData)?.get_flags_u8())
}

///	This command is used to start the process of discovering remote
///	devices using the limited discovery procedure. A Device Found event
///	will be sent for each discovered device.
///
///	The limited discovery uses active scanning for Low Energy scanning
///	and will search for devices with the limited discoverability flag
///	configured. On BR/EDR it uses LIAC and filters on the limited
///	discoverability flag of the class of device.
///
///	When the discovery procedure starts the Discovery event will
///	notify this similar to Start Discovery.
///
///	This command can only be used when the controller is powered.
pub async fn start_limited_discovery(
    socket: &mut ManagementStream,
    controller: Controller,
    address_types: BitFlags<AddressTypeFlag>,
    event_tx: Option<mpsc::Sender<Response>>,
) -> Result<BitFlags<AddressTypeFlag>> {
    let mut param = BytesMut::with_capacity(1);
    param.put_u8(address_types.bits());

    let (_, param) = exec_command(
        socket,
        Command::StartLimitedDiscovery,
        controller,
        Some(param.freeze()),
        event_tx,
    )
    .await?;

    Ok(param.ok_or(Error::NoData)?.get_flags_u8())
}
