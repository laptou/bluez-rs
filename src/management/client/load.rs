use super::*;
use crate::AddressType;

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
    socket: &mut ManagementStream,
    controller: Controller,
    keys: Vec<LinkKey>,
    debug: bool,
    event_tx: Option<mpsc::Sender<Response>>,
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

    let (_, _param) = exec_command(
        socket,
        Command::LoadLinkKeys,
        controller,
        Some(param.freeze()),
        event_tx,
    )
    .await?;

    Ok(())
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
    socket: &mut ManagementStream,
    controller: Controller,
    keys: Vec<LongTermKey>,
    event_tx: Option<mpsc::Sender<Response>>,
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

    let (_, _param) = exec_command(
        socket,
        Command::LoadLongTermKeys,
        controller,
        Some(param.freeze()),
        event_tx,
    )
    .await?;

    Ok(())
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
    socket: &mut ManagementStream,
    controller: Controller,
    keys: Vec<IdentityResolvingKey>,
    event_tx: Option<mpsc::Sender<Response>>,
) -> Result<()> {
    let mut param = BytesMut::with_capacity(2 + keys.len() * 23);
    param.put_u16_le(keys.len() as u16);

    for key in keys {
        param.put_slice(key.address.as_ref());
        param.put_u8(key.address_type as u8);
        param.put_slice(&key.value[..]);
    }

    let (_, _param) = exec_command(
        socket,
        Command::LoadIdentityResolvingKeys,
        controller,
        Some(param.freeze()),
        event_tx,
    )
    .await?;

    Ok(())
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
    socket: &mut ManagementStream,
    controller: Controller,
    connection_params: Vec<ConnectionParams>,
    event_tx: Option<mpsc::Sender<Response>>,
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

    let (_, _param) = exec_command(
        socket,
        Command::LoadConnectionParameters,
        controller,
        Some(param.freeze()),
        event_tx,
    )
    .await?;

    Ok(())
}

/// This command is used to feed the kernel a list of keys that
///	are known to be vulnerable.
///
///	If the pairing procedure produces any of these keys, they will be
///	silently dropped and any attempt to enable encryption rejected.
///
/// This command can be used when the controller is not powered.
pub async fn load_blocked_keys(
    socket: &mut ManagementStream,
    controller: Controller,
    keys: Vec<BlockedKey>,
    event_tx: Option<mpsc::Sender<Response>>,
) -> Result<()> {
    let mut param = BytesMut::with_capacity(2 + keys.len() * 17);
    param.put_u16_le(keys.len() as u16);

    for key in keys {
        param.put_u8(key.key_type as u8);
        param.put_slice(&key.value[..]);
    }

    let (_, _param) = exec_command(
        socket,
        Command::LoadBlockedKeys,
        controller,
        Some(param.freeze()),
        event_tx,
    )
    .await?;

    Ok(())
}

#[derive(Debug)]
pub struct LinkKey {
    pub address: Address,
    pub address_type: AddressType,
    pub key_type: LinkKeyType,
    pub value: [u8; 16],
    pub pin_length: u8,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, FromPrimitive)]
#[repr(u8)]
pub enum LinkKeyType {
    Combination = 0x00,
    LocalUnit = 0x01,
    RemoteUnit = 0x02,
    DebugCombination = 0x03,
    UnauthenticatedCombinationP192 = 0x04,
    AuthenticatedCombinationP192 = 0x05,
    ChangedCombination = 0x06,
    UnauthenticatedCombinationP256 = 0x07,
    AuthenticatedCombinationP256 = 0x08,
}

#[derive(Debug)]
pub struct LongTermKey {
    pub address: Address,
    pub address_type: AddressType,
    pub key_type: LongTermKeyType,
    pub master: u8,
    pub encryption_size: u8,
    pub encryption_diversifier: u16,
    pub random_number: u64,
    pub value: [u8; 16],
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, FromPrimitive)]
#[repr(u8)]
pub enum LongTermKeyType {
    UnauthenticatedLegacy = 0x00,
    AuthenticatedLegacy,
    UnauthenticatedP256,
    AuthenticatedP256,
    DebugP256,
}

#[derive(Debug)]
pub struct IdentityResolvingKey {
    pub address: Address,
    pub address_type: AddressType,
    pub value: [u8; 16],
}

pub struct BlockedKey {
    pub key_type: BlockedKeyType,
    pub value: [u8; 16],
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, FromPrimitive)]
pub enum BlockedKeyType {
    LinkKey = 1 << 0,
    LongTermKey = 1 << 1,
    IdentityResolvingKey = 1 << 2,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, FromPrimitive)]
#[repr(u8)]
pub enum SignatureResolvingKeyType {
    UnauthenticatedLocalCSRK = 0x00,
    UnauthenticatedRemoteCSRK = 0x01,
    AuthenticatedLocalCSRK = 0x02,
    AuthenticatedRemoteCSRK,
}
