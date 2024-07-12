use core::marker::PhantomData;

use contexts::storage_cache::StorageCache;
use farm_base_impl::base_traits_impl::FarmContract;

use crate::wrapped_farm_attributes::{throw_not_implemented_error, WrappedFarmAttributes};

multiversx_sc::imports!();

pub trait ScTraits:
    auto_farm::whitelists::farms_whitelist::FarmsWhitelistModule
    + auto_farm::external_storage_read::farm_storage_read::FarmStorageReadModule
    + rewards::RewardsModule
    + config::ConfigModule
    + token_send::TokenSendModule
    + farm_token::FarmTokenModule
    + pausable::PausableModule
    + permissions_module::PermissionsModule
    + multiversx_sc_modules::default_issue_callbacks::DefaultIssueCallbacksModule
    + farm_base_impl::base_farm_validation::BaseFarmValidationModule
    + farm_base_impl::enter_farm::BaseEnterFarmModule
    + utils::UtilsModule
    + crate::reward_tokens::RewardTokensModule
{
}

impl<T> ScTraits for T where
    T: auto_farm::whitelists::farms_whitelist::FarmsWhitelistModule
        + auto_farm::external_storage_read::farm_storage_read::FarmStorageReadModule
        + rewards::RewardsModule
        + config::ConfigModule
        + token_send::TokenSendModule
        + farm_token::FarmTokenModule
        + pausable::PausableModule
        + permissions_module::PermissionsModule
        + multiversx_sc_modules::default_issue_callbacks::DefaultIssueCallbacksModule
        + farm_base_impl::base_farm_validation::BaseFarmValidationModule
        + farm_base_impl::enter_farm::BaseEnterFarmModule
        + utils::UtilsModule
        + crate::reward_tokens::RewardTokensModule
{
}

pub struct BaseFarmLogicWrapper<T: ScTraits> {
    _phantom: PhantomData<T>,
}

impl<T> FarmContract for BaseFarmLogicWrapper<T>
where
    T: ScTraits,
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

    fn mint_per_block_rewards(
        sc: &Self::FarmSc,
        token_id: &TokenIdentifier<<Self::FarmSc as ContractBase>::Api>,
    ) -> BigUint<<Self::FarmSc as ContractBase>::Api> {
        let current_block_nonce = sc.blockchain().get_block_nonce();
        let last_reward_nonce = sc.last_reward_block_nonce().get();
        if current_block_nonce > last_reward_nonce {
            let to_mint =
                Self::calculate_per_block_rewards(sc, current_block_nonce, last_reward_nonce);
            if to_mint != 0 {
                Self::mint_rewards(sc, token_id, &to_mint);
            }

            to_mint
        } else {
            BigUint::zero()
        }
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
        let call_value = sc.call_value().single_esdt();

        WrappedFarmAttributes {
            farm_token_id: call_value.token_identifier,
            farm_token_nonce: call_value.token_nonce,
            reward_per_share: current_reward_per_share,
            creation_block: sc.blockchain().get_block_nonce(),
            current_token_amount: farming_token_amount,
        }
    }

    fn create_claim_rewards_initial_attributes(
        sc: &Self::FarmSc,
        _caller: ManagedAddress<<Self::FarmSc as ContractBase>::Api>,
        first_token_attributes: Self::AttributesType,
        current_reward_per_share: BigUint<<Self::FarmSc as ContractBase>::Api>,
    ) -> Self::AttributesType {
        WrappedFarmAttributes {
            farm_token_id: first_token_attributes.farm_token_id,
            farm_token_nonce: first_token_attributes.farm_token_nonce,
            reward_per_share: current_reward_per_share,
            creation_block: sc.blockchain().get_block_nonce(),
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
        throw_not_implemented_error::<<Self::FarmSc as ContractBase>::Api>();
    }
}
