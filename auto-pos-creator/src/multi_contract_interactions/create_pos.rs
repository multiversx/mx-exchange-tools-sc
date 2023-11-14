multiversx_sc::imports!();
multiversx_sc::derive_imports!();

use common_structs::PaymentsVec;

use crate::{
    common::payments_wrapper::PaymentsWrapper, configs::pairs_config::SwapOperationType,
    external_sc_interactions::pair_actions::PairTokenPayments,
};

pub type DoubleSwapResult<M> = PairTokenPayments<M>;

#[multiversx_sc::module]
pub trait CreatePosModule:
    crate::external_sc_interactions::pair_actions::PairActionsModule
    + crate::configs::pairs_config::PairsConfigModule
    + utils::UtilsModule
    + auto_farm::whitelists::farms_whitelist::FarmsWhitelistModule
    + auto_farm::whitelists::metastaking_whitelist::MetastakingWhitelistModule
    + auto_farm::external_storage_read::farm_storage_read::FarmStorageReadModule
    + auto_farm::external_storage_read::metastaking_storage_read::MetastakingStorageReadModule
    + crate::external_sc_interactions::egld_wrapper_actions::EgldWrapperActionsModule
    + crate::external_sc_interactions::router_actions::RouterActionsModule
    + crate::external_sc_interactions::farm_actions::FarmActionsModule
    + crate::external_sc_interactions::metastaking_actions::MetastakingActionsModule
{
    fn process_payment(
        &self,
        payment: EgldOrEsdtTokenPayment,
        swap_operations: MultiValueEncoded<SwapOperationType<Self::Api>>,
    ) -> EsdtTokenPayment {
        let esdt_payment = self.get_esdt_payment(payment);
        require!(esdt_payment.token_nonce == 0, "Only fungible ESDT accepted");

        if !swap_operations.is_empty() {
            self.call_router_swap(esdt_payment, swap_operations)
        } else {
            esdt_payment
        }
    }

    fn swap_half_input_payment(
        &self,
        first_payment: &mut EsdtTokenPayment,
        pair_address: ManagedAddress,
    ) -> EsdtTokenPayment {
        let pair_config = self.get_pair_config(&pair_address);

        let other_token_id = if first_payment.token_identifier == pair_config.first_token_id {
            pair_config.second_token_id
        } else if first_payment.token_identifier == pair_config.second_token_id {
            pair_config.first_token_id
        } else {
            sc_panic!("The output token identifier is not part of the LP")
        };

        let swap_input_payment = EsdtTokenPayment::new(
            first_payment.token_identifier.clone(),
            0,
            &first_payment.amount / 2u64,
        );
        first_payment.amount -= &swap_input_payment.amount;
        self.call_pair_swap(pair_address, swap_input_payment, other_token_id)
    }

    fn get_esdt_payment(&self, payment: EgldOrEsdtTokenPayment) -> EsdtTokenPayment {
        require!(payment.token_identifier.is_valid(), "Invalid payment");
        if payment.token_identifier.is_egld() {
            self.call_wrap_egld(payment.amount)
        } else {
            let esdt_payment = payment.unwrap_esdt();
            require!(esdt_payment.token_nonce == 0, "Invalid payment");
            esdt_payment
        }
    }

    fn split_payments(&self) -> (EgldOrEsdtTokenPayment, PaymentsVec<Self::Api>) {
        let raw_payments = self.call_value().any_payment();
        match raw_payments {
            EgldOrMultiEsdtPayment::Egld(egld_amount) => (
                EgldOrEsdtTokenPayment::new(
                    EgldOrEsdtTokenIdentifier::egld(),
                    0,
                    egld_amount.clone(),
                ),
                ManagedVec::new(),
            ),
            EgldOrMultiEsdtPayment::MultiEsdt(mut esdt_payments) => {
                require!(esdt_payments.len() > 0, "Invalid payments");
                let first_payment = self.pop_first_payment(&mut esdt_payments);

                (EgldOrEsdtTokenPayment::from(first_payment), esdt_payments)
            }
        }
    }

    fn create_lp_pos(
        &self,
        first_token_payment: EsdtTokenPayment,
        second_token_payment: EsdtTokenPayment,
        add_liq_first_token_min_amount_out: BigUint,
        add_liq_second_token_min_amount_out: BigUint,
        pair_address: ManagedAddress,
    ) -> (EsdtTokenPayment, PaymentsWrapper<Self::Api>) {
        let add_liq_result = self.call_pair_add_liquidity(
            pair_address,
            first_token_payment,
            second_token_payment,
            add_liq_first_token_min_amount_out,
            add_liq_second_token_min_amount_out,
        );

        let mut output_payments = PaymentsWrapper::new();
        output_payments.push(add_liq_result.first_tokens_remaining);
        output_payments.push(add_liq_result.second_tokens_remaining);

        (add_liq_result.lp_tokens, output_payments)
    }

    fn create_farm_pos(
        &self,
        first_token_payment: EsdtTokenPayment,
        second_token_payment: EsdtTokenPayment,
        add_liq_first_token_min_amount_out: BigUint,
        add_liq_second_token_min_amount_out: BigUint,
        pair_address: ManagedAddress,
        farm_address: ManagedAddress,
    ) -> (EsdtTokenPayment, PaymentsWrapper<Self::Api>) {
        let caller = self.blockchain().get_caller();

        let (lp_tokens, mut output_payments) = self.create_lp_pos(
            first_token_payment,
            second_token_payment,
            add_liq_first_token_min_amount_out,
            add_liq_second_token_min_amount_out,
            pair_address,
        );

        let enter_result = self.call_enter_farm(farm_address, caller.clone(), lp_tokens);
        output_payments.push(enter_result.rewards);

        (enter_result.new_farm_token, output_payments)
    }

    fn create_metastaking_pos(
        &self,
        first_token_payment: EsdtTokenPayment,
        second_token_payment: EsdtTokenPayment,
        add_liq_first_token_min_amount_out: BigUint,
        add_liq_second_token_min_amount_out: BigUint,
        pair_address: ManagedAddress,
        farm_address: ManagedAddress,
        metastaking_address: ManagedAddress,
    ) -> (EsdtTokenPayment, PaymentsWrapper<Self::Api>) {
        let caller = self.blockchain().get_caller();

        let (farm_tokens, mut output_payments) = self.create_farm_pos(
            first_token_payment,
            second_token_payment,
            add_liq_first_token_min_amount_out,
            add_liq_second_token_min_amount_out,
            pair_address,
            farm_address,
        );

        let stake_result = self.call_metastaking_stake(metastaking_address, caller, farm_tokens);

        output_payments.push(stake_result.staking_boosted_rewards);
        output_payments.push(stake_result.lp_farm_boosted_rewards);

        (stake_result.dual_yield_tokens, output_payments)
    }
}
