multiversx_sc::imports!();

use auto_pos_creator::configs::{self, pairs_config::SwapOperationType};
use common_structs::{Epoch, PaymentsVec};

use crate::create_pos;

pub struct AddLiquidityArguments<M: ManagedTypeApi> {
    pub payment: EsdtTokenPayment<M>,
    pub lock_epochs: Epoch,
    pub add_liq_first_token_min_amount: BigUint<M>,
    pub add_liq_second_token_min_amount: BigUint<M>,
}

#[multiversx_sc::module]
pub trait CreatePairPosModule:
    create_pos::CreatePosModule
    + crate::external_sc_interactions::egld_wrapper_actions::EgldWrapperActionsModule
    + crate::external_sc_interactions::energy_factory_actions::EnergyFactoryActionsModule
    + crate::external_sc_interactions::proxy_dex_actions::ProxyDexActionsModule
    + energy_query::EnergyQueryModule
    + utils::UtilsModule
    + configs::pairs_config::PairsConfigModule
    + auto_pos_creator::external_sc_interactions::pair_actions::PairActionsModule
    + auto_pos_creator::external_sc_interactions::router_actions::RouterActionsModule
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
            self.swap_half_input_payment(&mut first_token_payment, pair_address.clone());

        let (other_tokens, locked_tokens) = self.prepare_payments(
            lock_epochs,
            caller.clone(),
            first_token_payment,
            second_token_payment,
        );

        let (new_lp_tokens, mut output_payments) = self.create_lp_pos(
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

        let (new_lp_tokens, mut output_payments) = self.create_lp_pos(
            first_payment,
            second_payment,
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
