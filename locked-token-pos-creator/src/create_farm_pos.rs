multiversx_sc::imports!();
multiversx_sc::derive_imports!();

use auto_pos_creator::{
    configs::{self},
    external_sc_interactions::router_actions::SwapOperationType,
};
use common_structs::{Epoch, PaymentsVec};

#[multiversx_sc::module]
pub trait CreateFarmPosModule:
    configs::pairs_config::PairsConfigModule
    + utils::UtilsModule
    + energy_query::EnergyQueryModule
    + crate::external_sc_interactions::energy_factory_actions::EnergyFactoryActionsModule
    + crate::external_sc_interactions::proxy_dex_actions::ProxyDexActionsModule
    + crate::create_locked_pos::CreateLockedPosModule
    + crate::create_pair_pos::CreatePairPosModule
    + auto_pos_creator::multi_contract_interactions::create_pos::CreatePosModule
    + auto_pos_creator::external_sc_interactions::egld_wrapper_actions::EgldWrapperActionsModule
    + auto_pos_creator::external_sc_interactions::pair_actions::PairActionsModule
    + auto_pos_creator::external_sc_interactions::router_actions::RouterActionsModule
    + auto_pos_creator::external_sc_interactions::farm_actions::FarmActionsModule
    + auto_pos_creator::external_sc_interactions::metastaking_actions::MetastakingActionsModule
{
    #[payable("*")]
    #[endpoint(createFarmPosFromSingleToken)]
    fn create_farm_pos_from_single_token(
        &self,
        lock_epochs: Epoch,
        add_liq_first_token_min_amount: BigUint,
        add_liq_second_token_min_amount: BigUint,
        swap_operations: MultiValueEncoded<SwapOperationType<Self::Api>>,
    ) -> PaymentsVec<Self::Api> {
        let caller = self.blockchain().get_caller();
        let (first_payment, additional_payments) = self.split_first_payment();

        let pair_address = self.pair_address().get();
        let farm_address = self.farm_address().get();

        let mut first_token_payment = self.process_payment(first_payment, swap_operations);
        let second_token_payment =
            self.swap_half_input_payment_if_needed(&mut first_token_payment, pair_address.clone());

        let (other_tokens, locked_tokens) = self.prepare_locked_payments(
            lock_epochs,
            caller.clone(),
            first_token_payment,
            second_token_payment,
        );

        let (new_farm_tokens, mut output_payments) = self.create_locked_farm_pos(
            caller.clone(),
            other_tokens,
            locked_tokens,
            additional_payments,
            add_liq_first_token_min_amount,
            add_liq_second_token_min_amount,
            pair_address,
            farm_address,
        );
        output_payments.push(new_farm_tokens);

        output_payments.send_and_return(&caller)
    }

    #[payable("*")]
    #[endpoint(createFarmPosFromTwoTokens)]
    fn create_farm_pos_from_two_tokens(
        &self,
        add_liq_first_token_min_amount: BigUint,
        add_liq_second_token_min_amount: BigUint,
    ) -> PaymentsVec<Self::Api> {
        let caller = self.blockchain().get_caller();
        let (first_token_payment, second_token_payment, additional_payments) =
            self.split_first_two_payments();

        let pair_address = self.pair_address().get();
        let farm_address = self.farm_address().get();

        let (new_farm_tokens, mut output_payments) = self.create_locked_farm_pos(
            caller.clone(),
            first_token_payment,
            second_token_payment,
            additional_payments,
            add_liq_first_token_min_amount,
            add_liq_second_token_min_amount,
            pair_address,
            farm_address,
        );
        output_payments.push(new_farm_tokens);

        output_payments.send_and_return(&caller)
    }

    #[storage_mapper("wegldMexLpFarmAddress")]
    fn farm_address(&self) -> SingleValueMapper<ManagedAddress>;
}
