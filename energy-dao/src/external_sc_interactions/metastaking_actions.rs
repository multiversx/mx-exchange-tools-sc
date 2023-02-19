multiversx_sc::imports!();

use common_structs::PaymentsVec;
use farm_staking::unbond_farm::ProxyTrait as _;
use farm_staking_proxy::{
    proxy_actions::{
        claim::ProxyTrait as OtherProxyTrait3, stake::ProxyTrait as OtherProxyTrait,
        unstake::ProxyTrait as OtherProxyTrait2,
    },
    result_types::{ClaimDualYieldResult, StakeProxyResult, UnstakeResult},
};
use locked_token_wrapper::wrapped_token;

use super::energy_dao_config::{MetastakingState, WrappedMetastakingTokenAttributes};

#[multiversx_sc::module]
pub trait MetastakingActionsModule:
    crate::external_sc_interactions::energy_dao_config::EnergyDAOConfigModule
    + crate::external_sc_interactions::locked_token_actions::LockedTokenModule
    + utils::UtilsModule
    + energy_query::EnergyQueryModule
    + lkmex_transfer::energy_transfer::EnergyTransferModule
    + legacy_token_decode_module::LegacyTokenDecodeModule
    + wrapped_token::WrappedTokenModule
    + simple_lock::token_attributes::TokenAttributesModule
    + multiversx_sc_modules::default_issue_callbacks::DefaultIssueCallbacksModule
{
    fn call_enter_metastaking(
        &self,
        metastaking_address: ManagedAddress,
        lp_farm_tokens: PaymentsVec<Self::Api>,
    ) -> StakeProxyResult<Self::Api> {
        self.metastaking_proxy(metastaking_address)
            .stake_farm_tokens(OptionalValue::<ManagedAddress>::None)
            .with_multi_token_transfer(lp_farm_tokens)
            .execute_on_dest_context()
    }

    fn call_exit_metastaking(
        &self,
        metastaking_address: ManagedAddress,
        full_dual_yield_position: EsdtTokenPayment,
        exit_amount: BigUint,
    ) -> UnstakeResult<Self::Api> {
        self.metastaking_proxy(metastaking_address)
            .unstake_farm_tokens(
                BigUint::from(1u64), // pair_first_token_min_amount
                BigUint::from(1u64), // pair_second_token_min_amount
                exit_amount,
                OptionalValue::<ManagedAddress>::None,
            )
            .with_esdt_transfer(full_dual_yield_position)
            .execute_on_dest_context()
    }

    fn call_unbond_metastaking(
        &self,
        metastaking_address: ManagedAddress,
        unbond_position: EsdtTokenPayment,
    ) -> EsdtTokenPayment<Self::Api> {
        let staking_farm_address = self.get_staking_farm_address(&metastaking_address);
        self.farm_staking_proxy(staking_farm_address)
            .unbond_farm()
            .with_esdt_transfer(unbond_position)
            .execute_on_dest_context()
    }

    fn call_metastaking_claim(
        &self,
        ms_address: ManagedAddress,
        dual_yield_token: EsdtTokenPayment,
    ) -> ClaimDualYieldResult<Self::Api> {
        self.metastaking_proxy(ms_address)
            .claim_dual_yield_endpoint(OptionalValue::<ManagedAddress>::None)
            .with_esdt_transfer(dual_yield_token)
            .execute_on_dest_context()
    }

    fn update_metastaking_after_claim(
        &self,
        initial_metastaking_state: &MetastakingState<Self::Api>,
        metastaking_state_mapper: &mut SingleValueMapper<MetastakingState<Self::Api>>,
        new_dual_yield_token: &EsdtTokenPayment,
        lp_farm_rewards: EsdtTokenPayment,
        staking_rewards: EsdtTokenPayment,
        division_safety_constant: &BigUint,
    ) {
        let mut metastaking_state = metastaking_state_mapper.get();

        metastaking_state.ms_staked_value = new_dual_yield_token.amount.clone();
        metastaking_state.dual_yield_token_nonce = new_dual_yield_token.token_nonce;

        if lp_farm_rewards.amount == 0 && staking_rewards.amount == 0 {
            metastaking_state_mapper.set(metastaking_state);
            return;
        }

        let (lp_farm_rps_increase, staking_rps_increase) = self.compute_metastaking_rps_increase(
            &lp_farm_rewards.amount,
            &staking_rewards.amount,
            &new_dual_yield_token.amount,
            division_safety_constant,
        );
        let new_lp_farm_rewards = if initial_metastaking_state.lp_farm_reward_reserve > 0 {
            let mut reward_payments = ManagedVec::new();
            let current_farm_rewards = EsdtTokenPayment::new(
                lp_farm_rewards.token_identifier.clone(),
                initial_metastaking_state.lp_farm_reward_token_nonce,
                initial_metastaking_state.lp_farm_reward_reserve.clone(),
            );
            reward_payments.push(lp_farm_rewards);
            reward_payments.push(current_farm_rewards);
            self.merge_locked_tokens(reward_payments)
        } else {
            lp_farm_rewards
        };

        // lp_farm_reward_reserve increases by merging the new rewards with the old position
        metastaking_state.lp_farm_reward_reserve = new_lp_farm_rewards.amount;
        metastaking_state.lp_farm_reward_token_nonce = new_lp_farm_rewards.token_nonce;
        metastaking_state.staking_reward_reserve += staking_rewards.amount;
        metastaking_state.lp_farm_rps += lp_farm_rps_increase;
        metastaking_state.staking_rps += staking_rps_increase;

        metastaking_state_mapper.set(metastaking_state);
    }

    fn compute_metastaking_rps_increase(
        &self,
        lp_farm_reward: &BigUint,
        staking_reward: &BigUint,
        total_dual_yield_amount: &BigUint,
        division_safety_constant: &BigUint,
    ) -> (BigUint, BigUint) {
        if total_dual_yield_amount == &0u64 {
            return (BigUint::zero(), BigUint::zero());
        }

        let user_lp_farm_reward =
            (lp_farm_reward * division_safety_constant) / total_dual_yield_amount;
        let user_staking_reward =
            (staking_reward * division_safety_constant) / total_dual_yield_amount;
        (user_lp_farm_reward, user_staking_reward)
    }

    fn compute_user_metastaking_rewards(
        &self,
        metastaking_state_mapper: &mut SingleValueMapper<MetastakingState<Self::Api>>,
        payment: &EsdtTokenPayment,
        division_safety_constant: &BigUint,
    ) -> (BigUint, BigUint) {
        let metastaking_state = metastaking_state_mapper.get();
        let token_attributes: WrappedMetastakingTokenAttributes<Self::Api> =
            self.get_token_attributes(&payment.token_identifier, payment.token_nonce);
        let lp_farm_token_rps = token_attributes.lp_farm_token_rps;
        let staking_token_rps = token_attributes.staking_token_rps;

        let user_lp_farm_reward = if metastaking_state.lp_farm_rps > lp_farm_token_rps {
            let rps_diff = &metastaking_state.lp_farm_rps - &lp_farm_token_rps;
            &payment.amount * &rps_diff / division_safety_constant
        } else {
            BigUint::zero()
        };
        let user_staking_reward = if metastaking_state.staking_rps > staking_token_rps {
            let rps_diff = &metastaking_state.staking_rps - &staking_token_rps;
            &payment.amount * &rps_diff / division_safety_constant
        } else {
            BigUint::zero()
        };

        (user_lp_farm_reward, user_staking_reward)
    }

    #[proxy]
    fn metastaking_proxy(&self, sc_address: ManagedAddress)
        -> farm_staking_proxy::Proxy<Self::Api>;

    #[proxy]
    fn farm_staking_proxy(&self, sc_address: ManagedAddress) -> farm_staking::Proxy<Self::Api>;
}
