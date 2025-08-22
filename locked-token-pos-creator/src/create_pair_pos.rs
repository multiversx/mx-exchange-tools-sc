multiversx_sc::imports!();

use auto_pos_creator::{
    configs::{self},
    external_sc_interactions::router_actions::SwapOperationType,
};
use common_structs::{Epoch, PaymentsVec};

pub struct AddLiquidityArguments<M: ManagedTypeApi> {
    pub payment: EsdtTokenPayment<M>,
    pub lock_epochs: Epoch,
    pub add_liq_first_token_min_amount: BigUint<M>,
    pub add_liq_second_token_min_amount: BigUint<M>,
}

#[multiversx_sc::module]
pub trait CreatePairPosModule:
    configs::pairs_config::PairsConfigModule
    + utils::UtilsModule
    + read_external_storage::ReadExternalStorageModule
    + energy_query::EnergyQueryModule
    + crate::create_locked_pos::CreateLockedPosModule
    + crate::external_sc_interactions::energy_factory_actions::EnergyFactoryActionsModule
    + crate::external_sc_interactions::proxy_dex_actions::ProxyDexActionsModule
    + auto_pos_creator::multi_contract_interactions::create_pos::CreatePosModule
    + auto_pos_creator::external_sc_interactions::egld_wrapper_actions::EgldWrapperActionsModule
    + auto_pos_creator::external_sc_interactions::pair_actions::PairActionsModule
    + auto_pos_creator::external_sc_interactions::router_actions::RouterActionsModule
    + auto_pos_creator::external_sc_interactions::farm_actions::FarmActionsModule
    + auto_pos_creator::external_sc_interactions::metastaking_actions::MetastakingActionsModule
{
    #[payable("*")]
    #[endpoint(createPairPosFromSingleToken)]
    fn create_pair_pos_from_single_token_endpoint(
        &self,
        lock_epochs: Epoch,
        add_liq_first_token_min_amount: BigUint,
        add_liq_second_token_min_amount: BigUint,
        swap_operations: MultiValueEncoded<SwapOperationType<Self::Api>>,
    ) -> PaymentsVec<Self::Api> {
        let caller = self.blockchain().get_caller();
        let payment = self.call_value().egld_or_single_esdt();

        let pair_address = self.pair_address().get();
        let mut first_token_payment = self.process_payment(payment, swap_operations);
        let second_token_payment =
            self.swap_half_input_payment_if_needed(&mut first_token_payment, pair_address.clone());

        let (other_tokens, locked_tokens) = self.prepare_locked_payments(
            lock_epochs,
            caller.clone(),
            first_token_payment,
            second_token_payment,
        );

        let (new_lp_tokens, mut output_payments) = self.create_locked_lp_pos(
            other_tokens,
            locked_tokens,
            add_liq_first_token_min_amount,
            add_liq_second_token_min_amount,
            pair_address,
        );
        output_payments.push(new_lp_tokens);

        output_payments.send_and_return(&caller)
    }

    #[payable("*")]
    #[endpoint(createPairPosFromTwoTokens)]
    fn create_pair_pos_from_two_tokens_endpoint(
        &self,
        add_liq_first_token_min_amount: BigUint,
        add_liq_second_token_min_amount: BigUint,
    ) -> PaymentsVec<Self::Api> {
        let caller = self.blockchain().get_caller();
        let [first_payment, second_payment] = self.call_value().multi_esdt();

        let pair_address = self.pair_address().get();

        let (new_lp_tokens, mut output_payments) = self.create_locked_lp_pos(
            first_payment.clone(),
            second_payment.clone(),
            add_liq_first_token_min_amount,
            add_liq_second_token_min_amount,
            pair_address,
        );
        output_payments.push(new_lp_tokens);

        output_payments.send_and_return(&caller)
    }

    #[storage_mapper("wegldMexLpPairAddress")]
    fn pair_address(&self) -> SingleValueMapper<ManagedAddress>;
}
