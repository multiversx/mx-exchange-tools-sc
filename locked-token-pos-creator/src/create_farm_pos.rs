multiversx_sc::imports!();
multiversx_sc::derive_imports!();

use crate::create_pos;
use auto_pos_creator::configs::{self, pairs_config::SwapOperationType};
use common_structs::{Epoch, PaymentsVec};

#[multiversx_sc::module]
pub trait CreateFarmPosModule:
    create_pos::CreatePosModule
    + crate::external_sc_interactions::egld_wrapper_actions::EgldWrapperActionsModule
    + crate::external_sc_interactions::energy_factory_actions::EnergyFactoryActionsModule
    + crate::external_sc_interactions::proxy_dex_actions::ProxyDexActionsModule
    + crate::create_pair_pos::CreatePairPosModule
    + energy_query::EnergyQueryModule
    + utils::UtilsModule
    + configs::pairs_config::PairsConfigModule
    + auto_pos_creator::external_sc_interactions::pair_actions::PairActionsModule
    + auto_pos_creator::external_sc_interactions::router_actions::RouterActionsModule
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
        let payment = self.call_value().egld_or_single_esdt();

        let pair_address = self.pair_address().get();
        let farm_address = self.farm_address().get();

        let mut first_token_payment = self.process_payment(payment, swap_operations);
        let second_token_payment =
            self.swap_half_input_payment(&mut first_token_payment, pair_address.clone());

        let (other_tokens, locked_tokens) = self.prepare_payments(
            lock_epochs,
            caller.clone(),
            first_token_payment,
            second_token_payment,
        );

        let (new_farm_tokens, mut output_payments) = self.create_farm_pos(
            caller.clone(),
            other_tokens,
            locked_tokens,
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
        let [first_payment, second_payment] = self.call_value().multi_esdt();

        let pair_address = self.pair_address().get();
        let farm_address = self.farm_address().get();

        let (new_farm_tokens, mut output_payments) = self.create_farm_pos(
            caller.clone(),
            first_payment,
            second_payment,
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
