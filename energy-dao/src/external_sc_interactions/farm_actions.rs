multiversx_sc::imports!();

use common_structs::PaymentsVec;
use farm::{
    base_functions::ClaimRewardsResultType, EnterFarmResultType, ExitFarmWithPartialPosResultType,
};
use locked_token_wrapper::wrapped_token;

use super::farm_config::{FarmState, WrappedFarmTokenAttributes, MAX_PERCENT};

#[multiversx_sc::module]
pub trait FarmActionsModule:
    crate::external_sc_interactions::farm_config::FarmConfigModule
    + crate::external_sc_interactions::locked_token_actions::LockedTokenModule
    + utils::UtilsModule
    + energy_query::EnergyQueryModule
    + lkmex_transfer::energy_transfer::EnergyTransferModule
    + legacy_token_decode_module::LegacyTokenDecodeModule
    + wrapped_token::WrappedTokenModule
    + simple_lock::token_attributes::TokenAttributesModule
    + multiversx_sc_modules::default_issue_callbacks::DefaultIssueCallbacksModule
{
    fn call_enter_farm(
        &self,
        farm_address: ManagedAddress,
        farming_tokens: PaymentsVec<Self::Api>,
    ) -> EnterFarmResultType<Self::Api> {
        self.farm_proxy(farm_address)
            .enter_farm_endpoint(OptionalValue::<ManagedAddress>::None)
            .with_multi_token_transfer(farming_tokens)
            .execute_on_dest_context()
    }

    fn call_exit_farm(
        &self,
        farm_address: ManagedAddress,
        farm_tokens: EsdtTokenPayment,
    ) -> ExitFarmWithPartialPosResultType<Self::Api> {
        self.farm_proxy(farm_address)
            .exit_farm_endpoint(
                farm_tokens.amount.clone(),
                OptionalValue::<ManagedAddress>::None,
            )
            .with_esdt_transfer(farm_tokens)
            .execute_on_dest_context()
    }

    fn call_farm_claim(
        &self,
        farm_addr: ManagedAddress,
        farm_token: EsdtTokenPayment,
    ) -> ClaimRewardsResultType<Self::Api> {
        self.farm_proxy(farm_addr)
            .claim_rewards_endpoint(OptionalValue::<ManagedAddress>::None)
            .with_esdt_transfer(farm_token)
            .execute_on_dest_context()
    }

    fn update_farm_after_claim(
        &self,
        initial_farm_state: &FarmState<Self::Api>,
        farm_state_mapper: &mut SingleValueMapper<FarmState<Self::Api>>,
        new_farm_token: &EsdtTokenPayment,
        farm_rewards: EsdtTokenPayment,
        division_safety_constant: &BigUint,
    ) {
        let mut farm_state = farm_state_mapper.get();

        farm_state.farm_staked_value = new_farm_token.amount.clone();
        farm_state.farm_token_nonce = new_farm_token.token_nonce;

        if farm_rewards.amount == 0 {
            farm_state_mapper.set(farm_state);
            return;
        }

        let rps_increase = self.compute_farm_rps_increase(
            &farm_rewards.amount,
            &new_farm_token.amount,
            division_safety_constant,
        );
        let new_rewards = if initial_farm_state.reward_reserve > 0 {
            let mut reward_payments = ManagedVec::new();
            let current_rewards = EsdtTokenPayment::new(
                farm_rewards.token_identifier.clone(),
                initial_farm_state.reward_token_nonce,
                initial_farm_state.reward_reserve.clone(),
            );
            reward_payments.push(farm_rewards);
            reward_payments.push(current_rewards);
            self.merge_locked_tokens(reward_payments)
        } else {
            farm_rewards
        };

        farm_state.reward_token_nonce = new_rewards.token_nonce;
        farm_state.reward_reserve = new_rewards.amount;
        farm_state.farm_rps += rps_increase;

        farm_state_mapper.set(farm_state);
    }

    fn compute_farm_rps_increase(
        &self,
        reward: &BigUint,
        total_farm_amount: &BigUint,
        division_safety_constant: &BigUint,
    ) -> BigUint {
        if total_farm_amount != &0u64 {
            (reward * division_safety_constant) / total_farm_amount
        } else {
            BigUint::zero()
        }
    }

    fn apply_fee(&self, payment: &mut EsdtTokenPayment) {
        let penalty_percent = self.penalty_percent().get();
        let calculated_fee = &payment.amount * penalty_percent / MAX_PERCENT;

        let exit_fees_mapper = self.exit_fees();
        if exit_fees_mapper.is_empty() {
            let new_fee = EsdtTokenPayment::new(
                payment.token_identifier.clone(),
                payment.token_nonce,
                calculated_fee.clone(),
            );
            exit_fees_mapper.set(new_fee);
        } else {
            exit_fees_mapper.update(|fees| {
                if fees.token_identifier == payment.token_identifier {
                    fees.amount += &calculated_fee;
                }
            });
        }
        payment.amount -= calculated_fee;
    }

    fn compute_user_rewards_payment(
        &self,
        farm_state_mapper: &mut SingleValueMapper<FarmState<Self::Api>>,
        payment: &EsdtTokenPayment,
        division_safety_constant: &BigUint,
    ) -> EsdtTokenPayment {
        let farm_state = farm_state_mapper.get();
        let token_attributes: WrappedFarmTokenAttributes<Self::Api> =
            self.get_token_attributes(&payment.token_identifier, payment.token_nonce);
        let token_rps = token_attributes.token_rps;
        let reward = if farm_state.farm_rps > token_rps {
            let rps_diff = &farm_state.farm_rps - &token_rps;
            &payment.amount * &rps_diff / division_safety_constant
        } else {
            BigUint::zero()
        };
        let locked_token_id = self.get_locked_token_id();
        EsdtTokenPayment::new(locked_token_id, farm_state.reward_token_nonce, reward)
    }

    #[proxy]
    fn farm_proxy(&self, sc_address: ManagedAddress) -> farm_with_locked_rewards::Proxy<Self::Api>;
}
