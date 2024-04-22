multiversx_sc::imports!();

use common_structs::PaymentsVec;

use crate::common::payments_wrapper::PaymentsWrapper;

use super::exit_pos::{FarmExitArgs, MetastakingExitArgs, RemoveLiqArgs};

#[multiversx_sc::module]
pub trait ExitPosEndpointsModule:
    utils::UtilsModule
    + read_external_storage::ReadExternalStorageModule
    + crate::configs::pairs_config::PairsConfigModule
    + crate::external_sc_interactions::pair_actions::PairActionsModule
    + crate::external_sc_interactions::farm_actions::FarmActionsModule
    + crate::external_sc_interactions::metastaking_actions::MetastakingActionsModule
    + super::exit_pos::ExitPosModule
{
    #[payable("*")]
    #[endpoint(exitMetastakingPos)]
    fn exit_metastaking_pos_endpoint(
        &self,
        metastaking_address: ManagedAddress,
        first_token_min_amount_out: BigUint,
        second_token_min_amont_out: BigUint,
    ) -> PaymentsVec<Self::Api> {
        let caller = self.blockchain().get_caller();
        let payment = self.call_value().single_esdt();
        let mut output_payments = PaymentsWrapper::new();

        let args = MetastakingExitArgs {
            ms_address: metastaking_address,
            user: caller.clone(),
            ms_tokens: payment,
            first_token_min_amount_out,
            second_token_min_amont_out,
        };

        self.unstake_metastaking(&mut output_payments, args);

        output_payments.send_and_return(&caller)
    }

    #[payable("*")]
    #[endpoint(exitFarmPos)]
    fn exit_farm_pos(
        &self,
        farm_address: ManagedAddress,
        first_token_min_amount_out: BigUint,
        second_token_min_amont_out: BigUint,
    ) -> PaymentsVec<Self::Api> {
        let caller = self.blockchain().get_caller();
        let payment = self.call_value().single_esdt();
        let mut output_payments = PaymentsWrapper::new();

        let args = FarmExitArgs {
            farm_address,
            user: caller.clone(),
            farm_tokens: payment,
            first_token_min_amount_out,
            second_token_min_amont_out,
        };

        self.exit_farm(&mut output_payments, args);

        output_payments.send_and_return(&caller)
    }

    #[payable("*")]
    #[endpoint(exitLpPos)]
    fn exit_lp_pos(
        &self,
        pair_address: ManagedAddress,
        first_token_min_amount_out: BigUint,
        second_token_min_amont_out: BigUint,
    ) -> PaymentsVec<Self::Api> {
        let caller = self.blockchain().get_caller();
        let payment = self.call_value().single_esdt();
        let mut output_payments = PaymentsWrapper::new();

        let args = RemoveLiqArgs {
            pair_address,
            lp_tokens: payment,
            first_token_min_amount_out,
            second_token_min_amont_out,
        };

        self.remove_pair_liq(&mut output_payments, args);

        output_payments.send_and_return(&caller)
    }
}
