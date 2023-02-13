use common_structs::{FarmToken, FarmTokenAttributes, Nonce};
use fixed_supply_token::FixedSupplyToken;
use math::weighted_average_round_up;
use mergeable::Mergeable;

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

static NOT_IMPLEMENTED_ERR_MSG: &[u8] = b"Not implemented";

#[derive(TypeAbi, TopEncode, TopDecode, NestedEncode, NestedDecode, Clone, PartialEq, Debug)]
pub struct WrappedFarmAttributes<M: ManagedTypeApi> {
    pub farm_token: EsdtTokenPayment<M>,
    pub reward_per_share: BigUint<M>,
    pub creation_block: Nonce,
    pub current_token_amount: BigUint<M>,
}

impl<M: ManagedTypeApi> Mergeable<M> for WrappedFarmAttributes<M> {
    #[inline]
    fn can_merge_with(&self, other: &Self) -> bool {
        self.farm_token.token_identifier == other.farm_token.token_identifier
    }

    /// farm_token merging is done through an external call
    fn merge_with(&mut self, other: Self) {
        self.error_if_not_mergeable(&other);

        let first_supply = self.get_total_supply();
        let second_supply = other.get_total_supply();
        self.reward_per_share = weighted_average_round_up(
            self.reward_per_share.clone(),
            first_supply,
            other.reward_per_share.clone(),
            second_supply,
        );

        self.creation_block = core::cmp::max(self.creation_block, other.creation_block);
        self.current_token_amount += other.current_token_amount;
    }
}

impl<M: ManagedTypeApi> FixedSupplyToken<M> for WrappedFarmAttributes<M> {
    #[inline]
    fn get_total_supply(&self) -> BigUint<M> {
        self.current_token_amount.clone()
    }

    fn into_part(self, payment_amount: &BigUint<M>) -> Self {
        if payment_amount == &self.get_total_supply() {
            return self;
        }

        let new_current_token_amount = payment_amount.clone();
        let new_farm_token_amount = self.rule_of_three(payment_amount, &self.farm_token.amount);
        let partial_farm_token = EsdtTokenPayment::new(
            self.farm_token.token_identifier.clone(),
            self.farm_token.token_nonce,
            new_farm_token_amount,
        );

        WrappedFarmAttributes {
            farm_token: partial_farm_token,
            reward_per_share: self.reward_per_share,
            creation_block: self.creation_block,
            current_token_amount: new_current_token_amount,
        }
    }
}

/// only get_reward_per_share is being used
impl<M: ManagedTypeApi> FarmToken<M> for WrappedFarmAttributes<M> {
    #[inline]
    fn get_reward_per_share(&self) -> BigUint<M> {
        self.reward_per_share.clone()
    }

    #[inline]
    fn get_compounded_rewards(&self) -> BigUint<M> {
        BigUint::zero()
    }

    fn get_initial_farming_tokens(&self) -> BigUint<M> {
        throw_not_implemented_error::<M>();
    }
}

impl<M: ManagedTypeApi> From<FarmTokenAttributes<M>> for WrappedFarmAttributes<M> {
    fn from(_value: FarmTokenAttributes<M>) -> Self {
        throw_not_implemented_error::<M>();
    }
}

impl<M: ManagedTypeApi> Into<FarmTokenAttributes<M>> for WrappedFarmAttributes<M> {
    fn into(self) -> FarmTokenAttributes<M> {
        throw_not_implemented_error::<M>();
    }
}

pub fn throw_not_implemented_error<M: ManagedTypeApi>() -> ! {
    M::error_api_impl().signal_error(NOT_IMPLEMENTED_ERR_MSG)
}
