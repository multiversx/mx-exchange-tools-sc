use common_structs::PaymentsVec;

use crate::common::payments_wrapper::PaymentsWrapper;

use super::exit_pos::{ExitType, FarmExitArgs, MetastakingExitArgs, RemoveLiqArgs};

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait ExitPosEndpointsModule:
    crate::external_sc_interactions::pair_actions::PairActionsModule
    + crate::configs::pairs_config::PairsConfigModule
    + utils::UtilsModule
    + auto_farm::whitelists::farms_whitelist::FarmsWhitelistModule
    + auto_farm::whitelists::metastaking_whitelist::MetastakingWhitelistModule
    + auto_farm::external_storage_read::farm_storage_read::FarmStorageReadModule
    + auto_farm::external_storage_read::metastaking_storage_read::MetastakingStorageReadModule
    + crate::external_sc_interactions::farm_actions::FarmActionsModule
    + crate::external_sc_interactions::metastaking_actions::MetastakingActionsModule
    + super::exit_pos::ExitPosModule
{
    #[payable("*")]
    #[endpoint(fullExitPos)]
    fn full_exit_pos_endpoint(
        &self,
        first_token_min_amount_out: BigUint,
        second_token_min_amont_out: BigUint,
    ) -> PaymentsVec<Self::Api> {
        let caller = self.blockchain().get_caller();
        let payment = self.call_value().single_esdt();

        let exit_type = self.get_exit_type(&payment.token_identifier);
        let mut output_payments = PaymentsWrapper::new();

        match exit_type {
            ExitType::Metastaking(ms_addr) => {
                let args = MetastakingExitArgs {
                    ms_address: ms_addr,
                    user: caller.clone(),
                    ms_tokens: payment,
                    first_token_min_amount_out,
                    second_token_min_amont_out,
                };

                self.unstake_metastaking(&mut output_payments, args);
            }
            ExitType::Farm(farm_addr) => {
                let args = FarmExitArgs {
                    farm_address: farm_addr,
                    user: caller.clone(),
                    farm_tokens: payment,
                    first_token_min_amount_out,
                    second_token_min_amont_out,
                };

                self.exit_farm(&mut output_payments, args);
            }
            ExitType::Pair(pair_addr) => {
                let args = RemoveLiqArgs {
                    pair_address: pair_addr,
                    lp_tokens: payment,
                    first_token_min_amount_out,
                    second_token_min_amont_out,
                };

                self.remove_pair_liq(&mut output_payments, args);
            }
        };

        output_payments.send_and_return(&caller)
    }
}
