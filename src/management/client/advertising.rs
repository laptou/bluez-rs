use super::*;
use crate::util::BufExt;
use enumflags2::{bitflags, BitFlags};

///	This command is used to read the advertising features supported
///	by the controller and stack. The `max_adv_data_len` and `max_scan_rsp_len` provides extra
///	information about the maximum length of the data fields. For
///	now this will always return the value 31. Different flags
///	however might decrease the actual available length in these
///	data fields.
pub async fn get_advertising_features(
    socket: &mut ManagementStream,
    controller: Controller,
    event_tx: Option<mpsc::Sender<Response>>,
) -> Result<AdvertisingFeaturesInfo> {
    let (_, param) = exec_command(
        socket,
        Command::ReadAdvertisingFeatures,
        controller,
        None,
        event_tx,
    )
    .await?;

    let mut param = param.ok_or(Error::NoData)?;
    Ok(AdvertisingFeaturesInfo {
        supported_flags: param.get_flags_u32_le(),
        max_adv_data_len: param.get_u8(),
        max_scan_rsp_len: param.get_u8(),
        max_instances: param.get_u8(),
        instances: {
            let num_instances = param.get_u8() as usize;
            param.split_to(num_instances).to_vec()
        },
    })
}

///	This command is used to configure an advertising instance that
///	can be used to switch a Bluetooth Low Energy controller into
///	advertising mode.
///
///	Added advertising information with this command will not be visible
///	immediately if advertising is enabled via the Set Advertising
///	command. The usage of the Set Advertising command takes precedence
///	over this command. Instance information is stored and will be
///	advertised once advertising via Set Advertising has been disabled.
///
///	The Instance identifier is a value between 1 and the number of
///	supported instances. The value 0 is reserved.
/// When the connectable flag is set, then the controller will use
///	undirected connectable advertising. The value of the connectable
///	setting can be overwritten this way. This is useful to switch a
///	controller into connectable mode only for LE operation. This is
///	similar to the mode 0x02 from the Set Advertising command.
///
///	Secondary channel flags can be used to advertise in secondary
///	channel with the corresponding PHYs. These flag bits are mutually
///	exclusive and setting multiple will result in Invalid Parameter
///	error. Choosing either LE 1M or LE 2M will result in using
///	extended advertising on the primary channel with LE 1M and the
///	respectively LE 1M or LE 2M on the secondary channel. Choosing
///	LE Coded will result in using extended advertising on the primary
///	and secondary channels with LE Coded. Choosing none of these flags
///	will result in legacy advertising.
///
///	If only one advertising Instance has been added, then the `duration`
///	value will be ignored. It only applies for the case where multiple
///	Instances are configured. In that case every Instance will be
///	available for the `duration` time and after that it switches to
///	the next one. This is a simple round-robin based approach.
///
///	When a `timeout` is provided, then the `duration` subtracts from
///	the actual `timeout` value of that Instance. For example an Instance
///	with `timeout` of 5 and `duration` of 2 will be scheduled exactly 3
///	times, twice with 2 seconds and once with one second. Other
///	Instances have no influence on the `timeout`.
///
///	Re-adding an already existing instance (i.e. issuing the Add
///	Advertising command with an Instance identifier of an existing
///	instance) will update that instance's configuration.
///
///	An instance being added or changed while another instance is
///	being advertised will not be visible immediately but only when
///	the new/changed instance is being scheduled by the round robin
///	advertising algorithm.
///
///	Changes to an instance that is currently being advertised will
///	cancel that instance and switch to the next instance. The changes
///	will be visible the next time the instance is scheduled for
///	advertising. In case a single instance is active, this means
///	that changes will be visible right away.
///
///	A pre-requisite is that LE is already enabled, otherwise this
///	command will return a "rejected" response.
///
///	This command can be used when the controller is not powered and
///	all settings will be programmed once powered.
pub async fn add_advertising(
    socket: &mut ManagementStream,
    controller: Controller,
    info: AdvertisingParams,
    event_tx: Option<mpsc::Sender<Response>>,
) -> Result<u8> {
    let mut param = BytesMut::with_capacity(11 + info.adv_data.len() + info.scan_rsp.len());
    param.put_u8(info.instance);
    param.put_u32_le(info.flags.bits());
    param.put_u16_le(info.duration);
    param.put_u16_le(info.timeout);
    param.put_u8(info.adv_data.len() as u8);
    param.put_u8(info.scan_rsp.len() as u8);
    param.put_slice(&info.adv_data[..]);
    param.put_slice(&info.scan_rsp[..]);

    let (_, param) = exec_command(
        socket,
        Command::AddAdvertising,
        controller,
        Some(param.freeze()),
        event_tx,
    )
    .await?;

    Ok(param.ok_or(Error::NoData)?.get_u8())
}

///	This command is used to remove an advertising instance that
///	can be used to switch a Bluetooth Low Energy controller into
///	advertising mode.
///
///	When the `instance` parameter is zero, then all previously added
///	advertising Instances will be removed.
///
///	Removing advertising information with this command will not be
///	visible as long as advertising is enabled via the Set Advertising
///	command. The usage of the Set Advertising command takes precedence
///	over this command. Changes to Instance information are stored and
///	will be advertised once advertising via Set Advertising has been
///	disabled.
///
///	Removing an instance while it is being advertised will immediately
///	cancel the instance, even when it has been advertised less then its
///	configured Timeout or Duration.
///
///	This command can be used when the controller is not powered and
///	all settings will be programmed once powered.
pub async fn remove_advertising(
    socket: &mut ManagementStream,
    controller: Controller,
    instance: u8,
    event_tx: Option<mpsc::Sender<Response>>,
) -> Result<u8> {
    let mut param = BytesMut::with_capacity(1);
    param.put_u8(instance);

    let (_, param) = exec_command(
        socket,
        Command::RemoveAdvertising,
        controller,
        Some(param.freeze()),
        event_tx,
    )
    .await?;

    Ok(param.ok_or(Error::NoData)?.get_u8())
}

///	The Read Advertising Features command returns the overall maximum
///	size of advertising data and scan response data fields. That size is
///	valid when no Flags are used. However when certain Flags are used,
///	then the size might decrease. This command can be used to request
///	detailed information about the maximum available size.
///
///	To get accurate information about the available size, the same `flags`
///	values should be used with the Add Advertising command.
///
///	The `max_adv_data_len` and `max_scan_rsp_len` fields provide information
///	about the maximum length of the data fields for the given `flags`
///	values. When the `flags` field is zero, then these fields would contain
///	the same values as Read Advertising Features.
pub async fn get_advertising_size(
    socket: &mut ManagementStream,
    controller: Controller,
    instance: u8,
    event_tx: Option<mpsc::Sender<Response>>,
) -> Result<AdvertisingSizeInfo> {
    let mut param = BytesMut::with_capacity(1);
    param.put_u8(instance);

    let (_, param) = exec_command(
        socket,
        Command::GetAdvertisingSizeInfo,
        controller,
        Some(param.freeze()),
        event_tx,
    )
    .await?;

    let mut param = param.ok_or(Error::NoData)?;
    Ok(AdvertisingSizeInfo {
        instance: param.get_u8(),
        flags: param.get_flags_u32_le(),
        max_adv_data_len: param.get_u8(),
        max_scan_rsp_len: param.get_u8(),
    })
}

pub struct AdvertisingFeaturesInfo {
    pub supported_flags: BitFlags<AdvertisingFlags>,
    pub max_adv_data_len: u8,
    pub max_scan_rsp_len: u8,
    pub max_instances: u8,
    pub instances: Vec<u8>,
}

pub struct AdvertisingSizeInfo {
    pub instance: u8,
    pub flags: BitFlags<AdvertisingFlags>,
    pub max_adv_data_len: u8,
    pub max_scan_rsp_len: u8,
}

pub struct AdvertisingParams {
    pub instance: u8,

    ///	When the `EnterConnectable` flag is not set, then the controller will
    ///	use advertising based on the connectable setting. When using
    ///	non-connectable or scannable advertising, the controller will
    ///	be programmed with a non-resolvable random address. When the
    ///	system is connectable, then the identity address or resolvable
    ///	private address will be used.
    ///
    ///	Using the `EnterConnectable` flag is useful for peripheral mode support
    ///	where BR/EDR (and/or LE) is controlled by Add Device. This allows
    ///	making the peripheral connectable without having to interfere
    ///	with the global connectable setting.
    pub flags: BitFlags<AdvertisingFlags>,

    /// Configures the length of an Instance. The
    ///	value is in seconds.
    ///
    ///	A value of 0 indicates a default value is chosen for the
    ///	`duration`. The default is 2 seconds.
    pub duration: u16,

    /// Configures the life-time of an Instance. In
    ///	case the value 0 is used it indicates no expiration time. If a
    ///	timeout value is provided, then the advertising Instance will be
    ///	automatically removed when the timeout passes. The value for the
    ///	timeout is in seconds. Powering down a controller will invalidate
    ///	all advertising Instances and it is not possible to add a new
    ///	Instance with a timeout when the controller is powered down.
    pub timeout: u16,
    pub adv_data: Vec<u8>,

    ///	If `scan_rsp` is empty and connectable flag is not set and
    ///	the global connectable setting is off, then non-connectable
    ///	advertising is used. If `scan_rsp` is not empty
    ///	connectable flag is not set and the global advertising is off,
    ///	then scannable advertising is used. This small difference is
    ///	supported to provide less air traffic for devices implementing
    ///	broadcaster role.
    pub scan_rsp: Vec<u8>,
}

#[repr(u32)]
#[bitflags]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum AdvertisingFlags {
    /// Indicates support for connectable advertising
    ///	and for switching to connectable advertising independent of the
    ///	connectable global setting. When this flag is not supported, then
    ///	the global connectable setting determines if undirected connectable,
    ///	undirected scannable or undirected non-connectable advertising is
    ///	used. It also determines the use of non-resolvable random address
    ///	versus identity address or resolvable private address.
    EnterConnectable = 1 << 0,

    /// Indicates support for advertising with discoverable
    ///	mode enabled. Users of this flag will decrease the `max_adv_data_len`
    ///	by 3. In this case the advertising data flags are managed
    ///	and added in front of the provided advertising data.
    AdvertiseDiscoverable = 1 << 1,

    /// Indicates support for advertising with limited
    ///	discoverable mode enabled. Users of this flag will decrease the
    ///	`max_adv_data_len` by 3. In this case the advertising data
    ///	flags are managed and added in front of the provided advertising
    ///	data.
    AdvertiseLimitedDiscoverable = 1 << 2,

    /// Indicates support for automatically keeping the
    ///	Flags field of the advertising data updated. Users of this flag
    ///	will decrease the `max_adv_data_len` by 3 and need to keep
    ///	that in mind. The Flags field will be added in front of the
    ///	advertising data provided by the user. Note that with `AdvertiseDiscoverable`
    /// and `AdvertiseLimitedDiscoverable`, this one will be implicitly used even if it is
    ///	not marked as supported.
    AutoUpdateFlags = 1 << 3,

    /// Indicates support for automatically adding the
    ///	TX Power value to the advertising data. Users of this flag will
    ///	decrease the `max_adv_data_len` by 3. The `tx_power` field will
    ///	be added at the end of the user provided advertising data. If the
    ///	controller does not support TX Power information, then this bit will
    ///	not be set.
    AutoUpdateTxPower = 1 << 4,

    /// indicates support for automatically adding the
    ///	Appearance value to the scan response data. Users of this flag
    ///	will decrease the `max_scan_rsp_len` by 4. The `appearance`
    ///	field will be added in front of the scan response data provided
    ///	by the user. If the appearance value is not supported, then this
    ///	bit will not be set.
    AutoUpdateAppearance = 1 << 5,

    /// Indicates support for automatically adding the
    ///	Local Name value to the scan response data. This flag indicates
    ///	an opportunistic approach for the Local Name. If enough space
    ///	in the scan response data is available, it will be added. If the
    ///	space is limited a short version or no name information. The
    ///	Local Name will be added at the end of the scan response data.
    AutoUpdateLocalName = 1 << 6,

    /// Indicates support for advertising in secondary channel in LE 1M PHY.
    SecondaryChannelLE1M = 1 << 7,

    /// Indicates support for advertising in secondary channel in LE 2M PHY.
    /// Primary channel would be on 1M.
    SecondaryChannelLE2M = 1 << 8,

    /// Indicates support for advertising in secondary channel in LE CODED PHY.
    SecondaryChannelLECoded = 1 << 9,
}
