use super::*;

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
    socket: &mut ManagementStream,
    controller: Controller,
    device_class: DeviceClass,
    event_tx: Option<mpsc::Sender<Response>>,
) -> Result<(DeviceClass, ServiceClasses)> {
    let mut param = BytesMut::with_capacity(2);
    param.put_u16_le(device_class.into());

    let (_, param) = exec_command(
        socket,
        Command::SetDeviceClass,
        controller,
        Some(param.freeze()),
        event_tx,
    )
    .await?;

    Ok(device_class_from_bytes(param.ok_or(Error::NoData)?))
}

///	This command is used to add a UUID to be published in EIR data.
///	The accompanied SVC_Hint parameter is used to tell the kernel
///	whether the service class bits of the Class of Device value need
///	modifying due to this UUID.
///
///	This command can be used when the controller is not powered and
///	all settings will be programmed once powered.
///
///	In case the controller is powered off, `0x000000` will be returned
///	for the class of device parameter. And after power on the new
///	value will be announced via class of device changed event.
pub async fn add_uuid(
    socket: &mut ManagementStream,
    controller: Controller,
    uuid: [u8; 16],
    svc_hint: ServiceClasses,
    event_tx: Option<mpsc::Sender<Response>>,
) -> Result<(DeviceClass, ServiceClasses)> {
    let mut param = BytesMut::with_capacity(17);
    param.put_slice(&uuid[..]);
    param.put_u8((svc_hint.bits() >> 16) as u8);

    let (_, param) = exec_command(
        socket,
        Command::AddUUID,
        controller,
        Some(param.freeze()),
        event_tx,
    )
    .await?;

    Ok(device_class_from_bytes(param.ok_or(Error::NoData)?))
}

///	This command is used to remove a UUID previously added using the
///	Add UUID command.
///
///	When the UUID parameter is an empty UUID (16 x `0x00`), then all
///	previously loaded UUIDs will be removed.
///
///	This command can be used when the controller is not powered and
///	all settings will be programmed once powered.
///
///	In case the controller is powered off, `0x000000` will be returned
///	for the class of device parameter. And after power on the new
///	value will be announced via class of device changed event.
pub async fn remove_uuid(
    socket: &mut ManagementStream,
    controller: Controller,
    uuid: [u8; 16],
    event_tx: Option<mpsc::Sender<Response>>,
) -> Result<(DeviceClass, ServiceClasses)> {
    let param = BytesMut::from(&uuid[..]);

    let (_, param) = exec_command(
        socket,
        Command::RemoveUUID,
        controller,
        Some(param.freeze()),
        event_tx,
    )
    .await?;

    Ok(device_class_from_bytes(param.ok_or(Error::NoData)?))
}
