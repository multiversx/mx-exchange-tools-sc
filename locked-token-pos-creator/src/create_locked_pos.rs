multiversx_sc::imports!();
multiversx_sc::derive_imports!();

use auto_pos_creator::common::payments_wrapper::PaymentsWrapper;
use common_structs::{Epoch, PaymentsVec};

#[multiversx_sc::module]
pub trait CreateLockedPosModule:
    utils::UtilsModule
    + energy_query::EnergyQueryModule
    + crate::external_sc_interactions::proxy_dex_actions::ProxyDexActionsModule
    + crate::external_sc_interactions::energy_factory_actions::EnergyFactoryActionsModule
{
    fn prepare_locked_payments(
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

    fn create_locked_lp_pos(
        &self,
        first_token_payment: EsdtTokenPayment,
        second_token_payment: EsdtTokenPayment,
        add_liq_first_token_min_amount_out: BigUint,
        add_liq_second_token_min_amount_out: BigUint,
        pair_address: ManagedAddress,
    ) -> (EsdtTokenPayment, PaymentsWrapper<Self::Api>) {
        let mut output_payments = PaymentsWrapper::new();
        if second_token_payment.amount == 0 {
            return (first_token_payment, output_payments);
        }

        let mut payments = PaymentsVec::from_single_item(first_token_payment);
        payments.push(second_token_payment);
        let add_liq_result = self.call_add_liquidity_proxy(
            payments,
            pair_address,
            add_liq_first_token_min_amount_out,
            add_liq_second_token_min_amount_out,
        );

        output_payments.push(add_liq_result.wegld_leftover);
        output_payments.push(add_liq_result.locked_token_leftover);

        (add_liq_result.wrapped_lp_token, output_payments)
    }

    fn create_locked_farm_pos(
        &self,
        caller: ManagedAddress,
        first_token_payment: EsdtTokenPayment,
        second_token_payment: EsdtTokenPayment,
        additional_payments: PaymentsVec<Self::Api>,
        add_liq_first_token_min_amount_out: BigUint,
        add_liq_second_token_min_amount_out: BigUint,
        pair_address: ManagedAddress,
        farm_address: ManagedAddress,
    ) -> (EsdtTokenPayment, PaymentsWrapper<Self::Api>) {
        let (lp_tokens, mut output_payments) = self.create_locked_lp_pos(
            first_token_payment,
            second_token_payment,
            add_liq_first_token_min_amount_out,
            add_liq_second_token_min_amount_out,
            pair_address,
        );

        let mut payments = PaymentsVec::from_single_item(lp_tokens);
        payments.append_vec(additional_payments);
        let enter_result = self.call_enter_farm_proxy(caller, payments, farm_address);
        output_payments.push(enter_result.rewards);

        (enter_result.wrapped_farm_token, output_payments)
    }
}
