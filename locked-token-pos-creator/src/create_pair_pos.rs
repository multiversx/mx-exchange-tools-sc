use crate::external_sc_interactions::proxy_dex_actions::AddLiquidityProxyResult;

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait CreatePairPosModule:
    crate::external_sc_interactions::egld_wrapper_actions::EgldWrapperActionsModule
    + crate::external_sc_interactions::energy_factory_actions::EnergyFactoryActionsModule
    + crate::external_sc_interactions::pair_actions::PairActionsModule
    + crate::external_sc_interactions::proxy_dex_actions::ProxyDexActionsModule
    + energy_query::EnergyQueryModule
    + utils::UtilsModule
{
    /// lock_epochs must be one of the values allowed by energy_factory
    #[payable("*")]
    #[endpoint(createPairPosFromSingleToken)]
    fn create_pair_pos_from_single_token(
        &self,
        swap_min_amount_out: BigUint,
        lock_epochs: u64,
        add_liq_first_token_min_amount: BigUint,
        add_liq_second_token_min_amount: BigUint,
    ) -> AddLiquidityProxyResult<Self::Api> {
        let payment = self.call_value().egld_or_single_esdt();
        let wegld_token_id = self.wegld_token_id().get();
        let payment_esdt = if payment.token_identifier.is_egld() {
            self.call_wrap_egld(payment.amount)
        } else if payment.token_identifier == EgldOrEsdtTokenIdentifier::esdt(wegld_token_id) {
            payment.unwrap_esdt()
        } else {
            sc_panic!("Invalid payment");
        };

        let half_wegld_payment = EsdtTokenPayment::new(
            payment_esdt.token_identifier.clone(),
            0,
            payment_esdt.amount.clone() / 2u32,
        );
        let remaining_wegld = EsdtTokenPayment::new(
            payment_esdt.token_identifier.clone(),
            0,
            &payment_esdt.amount - &half_wegld_payment.amount,
        );

        let mex_token_id = self.get_base_token_id();
        let mex_tokens = self.call_pair_swap(half_wegld_payment, mex_token_id, swap_min_amount_out);

        let caller = self.blockchain().get_caller();
        let locked_tokens = self.call_lock_virtual(mex_tokens, lock_epochs, caller.clone());

        let mut proxy_payments = ManagedVec::new();
        proxy_payments.push(remaining_wegld);
        proxy_payments.push(locked_tokens);

        let pair_address = self.mex_wegld_pair_address().get();
        let add_liq_proxy_result = self.call_add_liquidity_proxy(
            proxy_payments,
            pair_address,
            add_liq_first_token_min_amount,
            add_liq_second_token_min_amount,
        );

        let mut output_payments =
            ManagedVec::from_single_item(add_liq_proxy_result.wrapped_token.clone());
        if add_liq_proxy_result.locked_token_leftover.amount > 0 {
            output_payments.push(add_liq_proxy_result.locked_token_leftover.clone());
        }
        if add_liq_proxy_result.wegld_leftover.amount > 0 {
            output_payments.push(add_liq_proxy_result.wegld_leftover.clone());
        }

        self.send().direct_multi(&caller, &output_payments);

        add_liq_proxy_result
    }
}
