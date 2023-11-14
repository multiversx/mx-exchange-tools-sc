use auto_pos_creator::{
    common::payments_wrapper::PaymentsWrapper, configs::pairs_config::SwapOperationType,
};
use common_structs::Epoch;

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[multiversx_sc::module]
pub trait CreatePosModule:
    utils::UtilsModule
    + energy_query::EnergyQueryModule
    + auto_pos_creator::configs::pairs_config::PairsConfigModule
    + crate::external_sc_interactions::egld_wrapper_actions::EgldWrapperActionsModule
    + crate::external_sc_interactions::energy_factory_actions::EnergyFactoryActionsModule
    + crate::external_sc_interactions::proxy_dex_actions::ProxyDexActionsModule
    + auto_pos_creator::external_sc_interactions::pair_actions::PairActionsModule
    + auto_pos_creator::external_sc_interactions::router_actions::RouterActionsModule
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

    fn prepare_payments(
        &self,
        lock_epochs: Epoch,
        caller: ManagedAddress,
        first_token_payment: EsdtTokenPayment,
        second_token_payment: EsdtTokenPayment,
    ) -> (EsdtTokenPayment, EsdtTokenPayment) {
        let base_token_id = self.get_base_token_id();
        if first_token_payment.token_identifier == base_token_id {
            let locked_tokens = self.call_lock_virtual(first_token_payment, lock_epochs, caller);
            (second_token_payment, locked_tokens)
        } else if second_token_payment.token_identifier == base_token_id {
            let locked_tokens = self.call_lock_virtual(second_token_payment, lock_epochs, caller);
            (first_token_payment, locked_tokens)
        } else {
            sc_panic!("Wrong payment tokens");
        }
    }

    fn create_lp_pos(
        &self,
        other_tokens: EsdtTokenPayment,
        locked_tokens: EsdtTokenPayment,
        add_liq_first_token_min_amount_out: BigUint,
        add_liq_second_token_min_amount_out: BigUint,
        pair_address: ManagedAddress,
    ) -> (EsdtTokenPayment, PaymentsWrapper<Self::Api>) {
        let mut proxy_payments = ManagedVec::new();
        proxy_payments.push(other_tokens);
        proxy_payments.push(locked_tokens);
        let add_liq_result = self.call_add_liquidity_proxy(
            proxy_payments,
            pair_address,
            add_liq_first_token_min_amount_out,
            add_liq_second_token_min_amount_out,
        );

        let mut output_payments = PaymentsWrapper::new();
        output_payments.push(add_liq_result.wegld_leftover);
        output_payments.push(add_liq_result.locked_token_leftover);

        (add_liq_result.wrapped_lp_token, output_payments)
    }

    fn create_farm_pos(
        &self,
        caller: ManagedAddress,
        other_tokens: EsdtTokenPayment,
        locked_tokens: EsdtTokenPayment,
        add_liq_first_token_min_amount: BigUint,
        add_liq_second_token_min_amount: BigUint,
        pair_address: ManagedAddress,
        farm_address: ManagedAddress,
    ) -> (EsdtTokenPayment, PaymentsWrapper<Self::Api>) {
        let (new_lp_tokens, mut output_payments) = self.create_lp_pos(
            other_tokens,
            locked_tokens,
            add_liq_first_token_min_amount,
            add_liq_second_token_min_amount,
            pair_address,
        );

        let enter_result = self.call_enter_farm_proxy(caller, new_lp_tokens, farm_address);
        output_payments.push(enter_result.rewards);

        (enter_result.wrapped_farm_token, output_payments)
    }
}
