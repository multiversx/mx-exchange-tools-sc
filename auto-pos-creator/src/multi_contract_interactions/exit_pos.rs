multiversx_sc::imports!();

use crate::common::payments_wrapper::PaymentsWrapper;

pub struct MetastakingExitArgs<M: ManagedTypeApi> {
    pub ms_address: ManagedAddress<M>,
    pub user: ManagedAddress<M>,
    pub ms_tokens: EsdtTokenPayment<M>,
    pub first_token_min_amount_out: BigUint<M>,
    pub second_token_min_amont_out: BigUint<M>,
}

pub struct FarmExitArgs<M: ManagedTypeApi> {
    pub farm_address: ManagedAddress<M>,
    pub user: ManagedAddress<M>,
    pub farm_tokens: EsdtTokenPayment<M>,
    pub first_token_min_amount_out: BigUint<M>,
    pub second_token_min_amont_out: BigUint<M>,
}

pub struct RemoveLiqArgs<M: ManagedTypeApi> {
    pub pair_address: ManagedAddress<M>,
    pub lp_tokens: EsdtTokenPayment<M>,
    pub first_token_min_amount_out: BigUint<M>,
    pub second_token_min_amont_out: BigUint<M>,
}

#[multiversx_sc::module]
pub trait ExitPosModule:
    crate::external_sc_interactions::pair_actions::PairActionsModule
    + crate::configs::pairs_config::PairsConfigModule
    + utils::UtilsModule
    + auto_farm::whitelists::farms_whitelist::FarmsWhitelistModule
    + auto_farm::whitelists::metastaking_whitelist::MetastakingWhitelistModule
    + auto_farm::external_storage_read::farm_storage_read::FarmStorageReadModule
    + auto_farm::external_storage_read::metastaking_storage_read::MetastakingStorageReadModule
    + crate::external_sc_interactions::farm_actions::FarmActionsModule
    + crate::external_sc_interactions::metastaking_actions::MetastakingActionsModule
{
    fn unstake_metastaking(
        &self,
        output_payments: &mut PaymentsWrapper<Self::Api>,
        args: MetastakingExitArgs<Self::Api>,
    ) {
        let unstake_result = self.call_metastaking_unstake(
            args.ms_address,
            args.user,
            args.ms_tokens,
            args.first_token_min_amount_out,
            args.second_token_min_amont_out,
        );
        output_payments.push(unstake_result.other_token_payment);
        output_payments.push(unstake_result.lp_farm_rewards);
        output_payments.push(unstake_result.staking_rewards);
        output_payments.push(unstake_result.unbond_staking_farm_token);
    }

    fn exit_farm(
        &self,
        output_payments: &mut PaymentsWrapper<Self::Api>,
        args: FarmExitArgs<Self::Api>,
    ) {
        let pair_address = self
            .pair_contract_address()
            .get_from_address(&args.farm_address);
        self.require_sc_address(&pair_address);

        let exit_farm_result = self.call_exit_farm(args.farm_address, args.user, args.farm_tokens);
        output_payments.push(exit_farm_result.rewards);

        let lp_tokens = exit_farm_result.farming_tokens;

        let pair_args = RemoveLiqArgs {
            pair_address,
            lp_tokens,
            first_token_min_amount_out: args.first_token_min_amount_out,
            second_token_min_amont_out: args.second_token_min_amont_out,
        };

        self.remove_pair_liq(output_payments, pair_args);
    }

    fn remove_pair_liq(
        &self,
        output_payments: &mut PaymentsWrapper<Self::Api>,
        args: RemoveLiqArgs<Self::Api>,
    ) {
        let remove_liq_result = self.call_pair_remove_liquidity(
            args.pair_address,
            args.lp_tokens,
            args.first_token_min_amount_out,
            args.second_token_min_amont_out,
        );
        output_payments.push(remove_liq_result.first_tokens);
        output_payments.push(remove_liq_result.second_tokens);
    }
}
