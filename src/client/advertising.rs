use super::*;
use crate::util::BufExt2;

impl<'a> BlueZClient<'a> {
    ///	This command is used to read the advertising features supported
    ///	by the controller and stack. The `max_adv_data_len` and `max_scan_rsp_len` provides extra
    ///	information about the maximum length of the data fields. For
    ///	now this will always return the value 31. Different flags
    ///	however might decrease the actual available length in these
    ///	data fields.
    pub async fn get_advertising_features(
        &mut self,
        controller: Controller,
    ) -> Result<AdvertisingFeaturesInfo> {
        self.exec_command(
            Command::ReadAdvertisingFeatures,
            controller,
            None,
            |_, param| {
                let mut param = param.unwrap();
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
            },
        )
        .await
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
        &mut self,
        controller: Controller,
        info: AdvertisingParams,
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

        self.exec_command(
            Command::AddAdvertising,
            controller,
            Some(param.to_bytes()),
            |_, param| Ok(param.unwrap().get_u8()),
        )
        .await
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
    pub async fn remove_advertising(&mut self, controller: Controller, instance: u8) -> Result<u8> {
        let mut param = BytesMut::with_capacity(1);
        param.put_u8(instance);

        self.exec_command(
            Command::RemoveAdvertising,
            controller,
            Some(param.to_bytes()),
            |_, param| Ok(param.unwrap().get_u8()),
        )
        .await
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
        &mut self,
        controller: Controller,
        instance: u8,
    ) -> Result<AdvertisingSizeInfo> {
        let mut param = BytesMut::with_capacity(1);
        param.put_u8(instance);

        self.exec_command(
            Command::GetAdvertisingSizeInfo,
            controller,
            Some(param.to_bytes()),
            |_, param| {
                let mut param = param.unwrap();
                Ok(AdvertisingSizeInfo {
                    instance: param.get_u8(),
                    flags: param.get_flags_u32_le(),
                    max_adv_data_len: param.get_u8(),
                    max_scan_rsp_len: param.get_u8(),
                })
            },
        )
        .await
    }
}
