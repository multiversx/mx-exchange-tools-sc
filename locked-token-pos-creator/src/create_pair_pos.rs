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
        let esdt_payment = self.get_esdt_payment(payment);
        let args = AddLiquidityArguments {
            payment: esdt_payment,
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

    fn get_or_swap_to_wegld(&self, payment: EsdtTokenPayment) -> EsdtTokenPayment {
        let wegld_token_id = self.wegld_token_id().get();
        if payment.token_identifier == wegld_token_id {
            return payment;
        }

        let pair_address = self
            .get_pair_address_for_tokens(&wegld_token_id, &payment.token_identifier)
            .unwrap_address();
        self.call_pair_swap(pair_address, payment, wegld_token_id, BigUint::from(1u64))
    }

    fn create_pair_pos_from_single_token(
        &self,
        args: AddLiquidityArguments<Self::Api>,
    ) -> AddLiquidityProxyResult<Self::Api> {
        let caller = self.blockchain().get_caller();
        let mex_token_id = self.get_base_token_id();
        let wegld_token_id = self.wegld_token_id().get();
        let mex_pair_address = self
            .get_pair_address_for_tokens(&wegld_token_id, &mex_token_id)
            .unwrap_address();

        let (half_wegld_payment_amount, mex_tokens_amount_to_lock) =
            if args.payment.token_identifier == mex_token_id {
                let half_payment = args.payment.amount.clone() / 2u32;
                let mex_tokens_to_swap = EsdtTokenPayment::new(
                    args.payment.token_identifier.clone(),
                    args.payment.token_nonce,
                    half_payment.clone(),
                );
                let half_wegld_payment = self.call_pair_swap(
                    mex_pair_address.clone(),
                    mex_tokens_to_swap,
                    wegld_token_id.clone(),
                    args.swap_min_amount_out,
                );
                let mex_tokens_amount_to_lock = args.payment.amount - half_payment;
                (half_wegld_payment.amount, mex_tokens_amount_to_lock)
            } else {
                let mut wegld_to_swap = self.get_or_swap_to_wegld(args.payment);
                let half_wegld_payment_amount = wegld_to_swap.amount.clone() / 2u32;
                wegld_to_swap.amount -= half_wegld_payment_amount.clone();

                let mex_tokens_to_lock = self.call_pair_swap(
                    mex_pair_address.clone(),
                    wegld_to_swap,
                    mex_token_id.clone(),
                    args.swap_min_amount_out,
                );
                (half_wegld_payment_amount, mex_tokens_to_lock.amount)
            };

        let half_wegld_payment =
            EsdtTokenPayment::new(wegld_token_id, 0, half_wegld_payment_amount);
        let mex_tokens_to_lock = EsdtTokenPayment::new(mex_token_id, 0, mex_tokens_amount_to_lock);
        let locked_tokens = self.call_lock_virtual(mex_tokens_to_lock, args.lock_epochs, caller);
        let mut proxy_payments = ManagedVec::new();
        proxy_payments.push(half_wegld_payment);
        proxy_payments.push(locked_tokens);

        self.call_add_liquidity_proxy(
            proxy_payments,
            mex_pair_address,
            args.add_liq_first_token_min_amount,
            args.add_liq_second_token_min_amount,
        )
    }
}
