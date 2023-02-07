multiversx_sc::imports!();

use crate::{
    common::errors::{
        ERROR_BAD_PAYMENT_TOKENS, ERROR_EXTERNAL_CONTRACT_OUTPUT, ERROR_FARM_DOES_NOT_EXIST,
        ERROR_UNBOND_TOO_SOON,
    },
    external_sc_interactions::farm_config::{UnstakeTokenAttributes, WrappedFarmTokenAttributes},
};
use common_structs::PaymentsVec;
use locked_token_wrapper::wrapped_token;

#[multiversx_sc::module]
pub trait FarmInteractionsModule:
    crate::external_sc_interactions::farm_config::FarmConfigModule
    + crate::external_sc_interactions::farm_actions::FarmActionsModule
    + crate::external_sc_interactions::locked_token_actions::LockedTokenModule
    + crate::external_sc_interactions::fees_collector_interactions::FeesCollectorInteractionsModule
    + lkmex_transfer::energy_transfer::EnergyTransferModule
    + legacy_token_decode_module::LegacyTokenDecodeModule
    + energy_query::EnergyQueryModule
    + token_send::TokenSendModule
    + utils::UtilsModule
    + wrapped_token::WrappedTokenModule
    + simple_lock::token_attributes::TokenAttributesModule
    + multiversx_sc_modules::default_issue_callbacks::DefaultIssueCallbacksModule
{
    #[payable("*")]
    #[endpoint(enterFarm)]
    fn enter_farm_endpoint(&self, farm_address: ManagedAddress) -> EsdtTokenPayment {
        let payment = self.call_value().single_esdt();
        let mut farm_config_mapper = self.farm_config(&farm_address);
        require!(!farm_config_mapper.is_empty(), ERROR_FARM_DOES_NOT_EXIST);
        let farm_config = farm_config_mapper.get();
        require!(
            farm_config.farming_token_id == payment.token_identifier,
            ERROR_BAD_PAYMENT_TOKENS
        );
        let mut enter_farm_payments = ManagedVec::new();
        enter_farm_payments.push(payment);

        let current_farm_position = EsdtTokenPayment::new(
            farm_config.farm_token_id.clone(),
            farm_config.farm_token_nonce,
            farm_config.farm_staked_value.clone(),
        );
        let initial_total_farm_amount = current_farm_position.amount.clone();
        if initial_total_farm_amount > 0 {
            enter_farm_payments.push(current_farm_position);
        }

        let enter_farm_result = self.call_enter_farm(farm_address, enter_farm_payments);
        let (new_farm_token, farm_rewards) = enter_farm_result.into_tuple();

        require!(
            new_farm_token.amount > initial_total_farm_amount,
            ERROR_EXTERNAL_CONTRACT_OUTPUT
        );

        let user_farm_amount = &new_farm_token.amount - &initial_total_farm_amount;
        self.update_farm_after_claim(
            &farm_config,
            &mut farm_config_mapper,
            new_farm_token,
            farm_rewards,
        );

        let caller = self.blockchain().get_caller();
        let new_farm_config = farm_config_mapper.get();
        let user_token_attributes = WrappedFarmTokenAttributes {
            token_rps: new_farm_config.farm_rps,
        };
        let output_payment = self.mint_tokens(
            farm_config.wrapped_token_id,
            user_farm_amount,
            &user_token_attributes,
        );
        self.send_payment_non_zero(&caller, &output_payment);

        output_payment
    }

    #[endpoint(claimFarmRewards)]
    fn claim_farm_rewards(&self, farm_address: ManagedAddress) {
        let mut farm_config_mapper = self.farm_config(&farm_address);
        require!(!farm_config_mapper.is_empty(), ERROR_FARM_DOES_NOT_EXIST);
        let farm_config = farm_config_mapper.get();

        let current_farm_position = EsdtTokenPayment::new(
            farm_config.farm_token_id.clone(),
            farm_config.farm_token_nonce,
            farm_config.farm_staked_value.clone(),
        );

        let claim_rewards_result = self.call_farm_claim(farm_address, current_farm_position);
        let (new_farm_token, farm_rewards) = claim_rewards_result.into_tuple();

        self.update_farm_after_claim(
            &farm_config,
            &mut farm_config_mapper,
            new_farm_token,
            farm_rewards,
        );
    }

    #[payable("*")]
    #[endpoint(claimUserRewards)]
    fn claim_user_rewards(&self, farm_address: ManagedAddress) {
        let payment = self.call_value().single_esdt();
        let mut farm_config_mapper = self.farm_config(&farm_address);
        require!(!farm_config_mapper.is_empty(), ERROR_FARM_DOES_NOT_EXIST);
        let farm_config = farm_config_mapper.get();
        require!(
            farm_config.wrapped_token_id == payment.token_identifier,
            ERROR_BAD_PAYMENT_TOKENS
        );
        let user_rewards = self.compute_user_rewards_payment(&mut farm_config_mapper, &payment);
        if user_rewards.amount > 0 {
            farm_config_mapper.update(|config| config.reward_reserve -= &user_rewards.amount);
        }
        let new_attributes = WrappedFarmTokenAttributes {
            token_rps: farm_config.farm_rps,
        };
        self.burn_tokens(
            &payment.token_identifier,
            payment.token_nonce,
            &payment.amount,
        );
        let new_farm_token = self.mint_tokens(
            farm_config.wrapped_token_id,
            payment.amount,
            &new_attributes,
        );

        let mut user_payments = ManagedVec::new();
        user_payments.push(new_farm_token);
        if user_rewards.amount > 0 {
            let wrapper_user_rewards = self.wrap_locked_token(user_rewards);
            user_payments.push(wrapper_user_rewards);
        }
        let caller = self.blockchain().get_caller();
        self.send().direct_multi(&caller, &user_payments);
    }

    #[payable("*")]
    #[endpoint(unstakeFarm)]
    fn unstake_farm(&self, farm_address: ManagedAddress) -> EsdtTokenPayment {
        let payment = self.call_value().single_esdt();
        let mut farm_config_mapper = self.farm_config(&farm_address);
        require!(!farm_config_mapper.is_empty(), ERROR_FARM_DOES_NOT_EXIST);
        let farm_config = farm_config_mapper.get();
        require!(
            farm_config.wrapped_token_id == payment.token_identifier,
            ERROR_BAD_PAYMENT_TOKENS
        );

        let current_farm_position = EsdtTokenPayment::new(
            farm_config.farm_token_id.clone(),
            farm_config.farm_token_nonce,
            farm_config.farm_staked_value.clone(),
        );
        let claim_rewards_result = self.call_farm_claim(farm_address, current_farm_position);
        let (new_farm_token, farm_rewards) = claim_rewards_result.into_tuple();
        self.update_farm_after_claim(
            &farm_config,
            &mut farm_config_mapper,
            new_farm_token.clone(),
            farm_rewards,
        );

        let user_rewards = self.compute_user_rewards_payment(&mut farm_config_mapper, &payment);

        farm_config_mapper.update(|config| {
            config.farm_staked_value -= &payment.amount;
            config.farm_unstaked_value += &payment.amount;
            if user_rewards.amount > 0 {
                config.reward_reserve -= &user_rewards.amount
            }
        });

        self.burn_tokens(
            &payment.token_identifier,
            payment.token_nonce,
            &payment.amount,
        );
        let current_epoch = self.blockchain().get_block_epoch();
        let unstake_attributes = UnstakeTokenAttributes {
            unstake_epoch: current_epoch,
            token_nonce: new_farm_token.token_nonce,
        };
        let unstake_token_payment = self.mint_tokens(
            farm_config.unstake_token_id.clone(),
            payment.amount,
            &unstake_attributes,
        );

        let caller = self.blockchain().get_caller();
        self.send_payment_non_zero(&caller, &unstake_token_payment);

        unstake_token_payment
    }

    #[payable("*")]
    #[endpoint(unbondFarm)]
    fn unbond_farm(&self, farm_address: ManagedAddress) -> PaymentsVec<Self::Api> {
        let payment = self.call_value().single_esdt();
        let farm_config_mapper = self.farm_config(&farm_address);
        require!(!farm_config_mapper.is_empty(), ERROR_FARM_DOES_NOT_EXIST);
        let farm_config = farm_config_mapper.get();
        require!(
            farm_config.unstake_token_id == payment.token_identifier,
            ERROR_BAD_PAYMENT_TOKENS
        );
        let unstake_attributes: UnstakeTokenAttributes =
            self.get_token_attributes(&payment.token_identifier, payment.token_nonce);
        let current_epoch = self.blockchain().get_block_epoch();
        let unbond_period = self.unbond_period().get();
        require!(
            current_epoch >= unstake_attributes.unstake_epoch + unbond_period,
            ERROR_UNBOND_TOO_SOON
        );

        let unstake_payment = EsdtTokenPayment::new(
            farm_config.farm_token_id,
            unstake_attributes.token_nonce,
            payment.amount.clone(),
        );
        let exit_farm_result = self.call_exit_farm(farm_address, unstake_payment);
        let (farming_tokens, locked_rewards_payment, _) = exit_farm_result.into_tuple();

        farm_config_mapper.update(|config| {
            config.farm_unstaked_value -= &payment.amount;
        });

        self.burn_tokens(
            &payment.token_identifier,
            payment.token_nonce,
            &payment.amount,
        );

        let remaining_payment = self.apply_fee(farming_tokens);
        let mut user_payments = ManagedVec::new();
        user_payments.push(remaining_payment);
        if locked_rewards_payment.amount > 0 {
            user_payments.push(locked_rewards_payment);
        }
        let caller = self.blockchain().get_caller();
        self.send().direct_multi(&caller, &user_payments);

        user_payments
    }
}
