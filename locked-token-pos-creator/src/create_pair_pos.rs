multiversx_sc::imports!();

use auto_pos_creator::configs;
use common_structs::Epoch;

use crate::external_sc_interactions::proxy_dex_actions::AddLiquidityProxyResult;

pub struct AddLiquidityArguments<M: ManagedTypeApi> {
    pub payment: EsdtTokenPayment<M>,
    pub swap_min_amount_out: BigUint<M>,
    pub lock_epochs: Epoch,
    pub add_liq_first_token_min_amount: BigUint<M>,
    pub add_liq_second_token_min_amount: BigUint<M>,
}

#[multiversx_sc::module]
pub trait CreatePairPosModule:
    crate::external_sc_interactions::egld_wrapper_actions::EgldWrapperActionsModule
    + crate::external_sc_interactions::energy_factory_actions::EnergyFactoryActionsModule
    + crate::external_sc_interactions::pair_actions::PairActionsModule
    + crate::external_sc_interactions::proxy_dex_actions::ProxyDexActionsModule
    + energy_query::EnergyQueryModule
    + utils::UtilsModule
    + configs::pairs_config::PairsConfigModule
{
    /// lock_epochs must be one of the values allowed by energy_factory
    #[payable("*")]
    #[endpoint(createPairPosFromSingleToken)]
    fn create_pair_pos_from_single_token_endpoint(
        &self,
        swap_min_amount_out: BigUint,
        lock_epochs: Epoch,
        add_liq_first_token_min_amount: BigUint,
        add_liq_second_token_min_amount: BigUint,
    ) -> AddLiquidityProxyResult<Self::Api> {
        let payment = self.call_value().egld_or_single_esdt();
        let wegld_payment = self.get_wegld_payment(payment);
        let args = AddLiquidityArguments {
            payment: wegld_payment,
            swap_min_amount_out,
            lock_epochs,
            add_liq_first_token_min_amount,
            add_liq_second_token_min_amount,
        };

        let add_liq_result = self.create_pair_pos_from_single_token(args);

        let mut output_payments =
            ManagedVec::from_single_item(add_liq_result.wrapped_lp_token.clone());
        if add_liq_result.locked_token_leftover.amount > 0 {
            output_payments.push(add_liq_result.locked_token_leftover.clone());
        }
        if add_liq_result.wegld_leftover.amount > 0 {
            output_payments.push(add_liq_result.wegld_leftover.clone());
        }

        let caller = self.blockchain().get_caller();
        self.send().direct_multi(&caller, &output_payments);

        add_liq_result
    }

    fn get_wegld_payment(&self, payment: EgldOrEsdtTokenPayment) -> EsdtTokenPayment {
        require!(payment.token_identifier.is_valid(), "Invalid payment");
        let wegld_token_id = self.wegld_token_id().get();
        if payment.token_identifier.is_egld() {
            return self.call_wrap_egld(payment.amount);
        };
        let esdt_payment = payment.unwrap_esdt();
        if esdt_payment.token_identifier == wegld_token_id {
            esdt_payment
        } else {
            let pair_address = self
                .get_pair_address_for_tokens(&wegld_token_id, &esdt_payment.token_identifier)
                .unwrap_address();
            let wegld_payment = self.call_pair_swap(
                pair_address,
                esdt_payment,
                wegld_token_id,
                BigUint::from(1u64),
            );
            wegld_payment
        }
    }

    fn create_pair_pos_from_single_token(
        &self,
        args: AddLiquidityArguments<Self::Api>,
    ) -> AddLiquidityProxyResult<Self::Api> {
        let half_wegld_payment = EsdtTokenPayment::new(
            args.payment.token_identifier.clone(),
            0,
            args.payment.amount.clone() / 2u32,
        );
        let remaining_wegld = EsdtTokenPayment::new(
            args.payment.token_identifier.clone(),
            0,
            &args.payment.amount - &half_wegld_payment.amount,
        );

        let mex_token_id = self.get_base_token_id();
        let wegld_token_id = self.wegld_token_id().get();
        let mex_pair_address = self
            .get_pair_address_for_tokens(&wegld_token_id, &mex_token_id)
            .unwrap_address();
        let mex_tokens = self.call_pair_swap(
            mex_pair_address.clone(),
            half_wegld_payment,
            mex_token_id,
            args.swap_min_amount_out,
        );

        let caller = self.blockchain().get_caller();
        let locked_tokens = self.call_lock_virtual(mex_tokens, args.lock_epochs, caller);

        let mut proxy_payments = ManagedVec::new();
        proxy_payments.push(remaining_wegld);
        proxy_payments.push(locked_tokens);

        self.call_add_liquidity_proxy(
            proxy_payments,
            mex_pair_address,
            args.add_liq_first_token_min_amount,
            args.add_liq_second_token_min_amount,
        )
    }
}
