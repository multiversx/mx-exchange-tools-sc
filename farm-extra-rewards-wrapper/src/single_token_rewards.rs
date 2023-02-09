use core::marker::PhantomData;

use contexts::storage_cache::StorageCache;
use farm_base_impl::base_traits_impl::FarmContract;

use crate::wrapped_farm_attributes::{WrappedFarmAttributes, NOT_IMPLEMENTED_ERR_MSG};
use crate::FarmExtraRewardsWrapper;

multiversx_sc::imports!();

pub struct Wrapper<T: FarmExtraRewardsWrapper> {
    _phantom: PhantomData<T>,
}

impl<T> FarmContract for Wrapper<T>
where
    T: FarmExtraRewardsWrapper,
{
    type FarmSc = T;
    type AttributesType = WrappedFarmAttributes<<Self::FarmSc as ContractBase>::Api>;

    #[inline]
    fn mint_rewards(
        _sc: &Self::FarmSc,
        _token_id: &TokenIdentifier<<Self::FarmSc as ContractBase>::Api>,
        _amount: &BigUint<<Self::FarmSc as ContractBase>::Api>,
    ) {
    }

    fn generate_aggregated_rewards(
        sc: &Self::FarmSc,
        storage_cache: &mut StorageCache<Self::FarmSc>,
    ) {
        let accumulated_rewards_mapper = sc.accumulated_rewards(&storage_cache.reward_token_id);
        let mut accumulated_rewards = accumulated_rewards_mapper.get();
        let reward_capacity = sc.reward_capacity(&storage_cache.reward_token_id).get();
        let remaining_rewards = &reward_capacity - &accumulated_rewards;

        let mut total_reward = Self::mint_per_block_rewards(sc, &storage_cache.reward_token_id);
        total_reward = core::cmp::min(total_reward, remaining_rewards);
        if total_reward == 0 {
            return;
        }

        storage_cache.reward_reserve += &total_reward;
        accumulated_rewards += &total_reward;
        accumulated_rewards_mapper.set(&accumulated_rewards);

        if storage_cache.farm_token_supply == 0 {
            return;
        }

        let increase = (&total_reward * &storage_cache.division_safety_constant)
            / &storage_cache.farm_token_supply;
        storage_cache.reward_per_share += &increase;
    }

    fn create_enter_farm_initial_attributes(
        sc: &Self::FarmSc,
        _caller: ManagedAddress<<Self::FarmSc as ContractBase>::Api>,
        farming_token_amount: BigUint<<Self::FarmSc as ContractBase>::Api>,
        current_reward_per_share: BigUint<<Self::FarmSc as ContractBase>::Api>,
    ) -> Self::AttributesType {
        WrappedFarmAttributes {
            farm_token: sc.call_value().single_esdt(),
            reward_per_share: current_reward_per_share,
            current_token_amount: farming_token_amount,
        }
    }

    fn create_claim_rewards_initial_attributes(
        _sc: &Self::FarmSc,
        _caller: ManagedAddress<<Self::FarmSc as ContractBase>::Api>,
        first_token_attributes: Self::AttributesType,
        current_reward_per_share: BigUint<<Self::FarmSc as ContractBase>::Api>,
    ) -> Self::AttributesType {
        WrappedFarmAttributes {
            farm_token: first_token_attributes.farm_token,
            reward_per_share: current_reward_per_share,
            current_token_amount: first_token_attributes.current_token_amount,
        }
    }

    fn create_compound_rewards_initial_attributes(
        _sc: &Self::FarmSc,
        _caller: ManagedAddress<<Self::FarmSc as ContractBase>::Api>,
        _first_token_attributes: Self::AttributesType,
        _current_reward_per_share: BigUint<<Self::FarmSc as ContractBase>::Api>,
        _reward: &BigUint<<Self::FarmSc as ContractBase>::Api>,
    ) -> Self::AttributesType {
        <Self::FarmSc as ContractBase>::Api::error_api_impl().signal_error(NOT_IMPLEMENTED_ERR_MSG);
    }
}
