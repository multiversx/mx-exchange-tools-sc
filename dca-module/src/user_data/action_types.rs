multiversx_sc::imports!();
multiversx_sc::derive_imports!();

pub type ActionId = u64;
pub type TotalActions = usize;
pub type NrRetries = usize;
pub type Timestamp = u64;
pub type GasLimit = u64;

/// Pairs of (pair address, endpoint name, requested token, min amount out)
pub type RouterSwapOperationType<M> =
    MultiValue4<ManagedAddress<M>, ManagedBuffer<M>, TokenIdentifier<M>, BigUint<M>>;

pub static SWAP_TOKENS_FIXED_INPUT_FUNC_NAME: &[u8] = b"swapTokensFixedInput";

#[derive(TypeAbi, TopEncode, TopDecode, NestedEncode, NestedDecode, ManagedVecItem)]
pub struct Action<M: ManagedTypeApi> {
    pub pair_address: ManagedAddress<M>,
    pub endpoint_name: ManagedBuffer<M>,
    pub requested_token: TokenIdentifier<M>,
    pub min_amount_out: BigUint<M>,
}

impl<M: ManagedTypeApi> From<RouterSwapOperationType<M>> for Action<M> {
    fn from(value: RouterSwapOperationType<M>) -> Self {
        let (pair_address, endpoint_name, requested_token, min_amount_out) = value.into_tuple();

        Self {
            pair_address,
            endpoint_name,
            requested_token,
            min_amount_out,
        }
    }
}

impl<M: ManagedTypeApi> From<Action<M>> for RouterSwapOperationType<M> {
    #[inline]
    fn from(value: Action<M>) -> Self {
        (
            value.pair_address,
            value.endpoint_name,
            value.requested_token,
            value.min_amount_out,
        )
            .into()
    }
}

pub type SwapOperationTypeUserArg<M> =
    MultiValue3<ManagedAddress<M>, TokenIdentifier<M>, BigUint<M>>;

// "From" trait can't be implemented for types not defined in this crate, so we need this workaround.
//
// Simple method doesn't work either, same problem.
pub fn router_arg_from_user_arg<M: ManagedTypeApi>(
    value: SwapOperationTypeUserArg<M>,
) -> RouterSwapOperationType<M> {
    let (pair_address, requested_token, min_amount_out) = value.into_tuple();

    (
        pair_address,
        SWAP_TOKENS_FIXED_INPUT_FUNC_NAME.into(),
        requested_token,
        min_amount_out,
    )
        .into()
}

#[derive(TypeAbi, TopEncode, TopDecode, NestedEncode, NestedDecode, PartialEq)]
pub enum TradeFrequency {
    Hourly,
    Daily,
    Weekly,
}

pub const HOURLY_TIMESTAMP: Timestamp = 60 * 60;
pub const DAILY_TIMESTAMP: Timestamp = 24 * HOURLY_TIMESTAMP;
pub const WEEKLY_TIMESTAMP: Timestamp = 7 * DAILY_TIMESTAMP;

impl TradeFrequency {
    // I don't think it's worth implementing From/Into, as that would make the code more unclear
    pub fn to_timestamp(&self) -> Timestamp {
        match *self {
            TradeFrequency::Hourly => HOURLY_TIMESTAMP,
            TradeFrequency::Daily => DAILY_TIMESTAMP,
            TradeFrequency::Weekly => WEEKLY_TIMESTAMP,
        }
    }
}

#[derive(TypeAbi, TopEncode, TopDecode)]
pub struct ActionInfo<M: ManagedTypeApi> {
    pub owner_id: AddressId,
    pub trade_frequency: TradeFrequency,
    pub last_action_timestamp: Timestamp,
    pub total_actions_left: TotalActions,
    pub action_in_progress: bool,
    pub actions: ManagedVec<M, Action<M>>,
}
