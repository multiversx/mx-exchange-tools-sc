use auto_farm::common::address_to_id_mapper::NULL_ID;
use common_structs::PaymentsVec;
use farm_staking_proxy::result_types::StakeProxyResult;

use crate::{
    common::payments_wrapper::PaymentsWrapper,
    external_sc_interactions::{
        farm_actions::EnterFarmResultWrapper, pair_actions::PairTokenPayments,
    },
};

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

pub type DoubleSwapResult<M> = PairTokenPayments<M>;

pub static COULD_NOT_CREATE_POS_ERR_MSG: &[u8] = b"Could not create position";
pub static UNKNOWN_PAIR_ERR_MSG: &[u8] = b"Unknown pair SC";

#[derive(TypeAbi, TopEncode, TopDecode)]
pub enum StepsToPerform {
    AddLiquidity,
    EnterFarm,
    EnterMetastaking,
}

pub struct CreatePosArgs<M: ManagedTypeApi> {
    pub caller: ManagedAddress<M>,
    pub dest_pair_address: ManagedAddress<M>,
    pub pair_input_tokens: PairTokenPayments<M>,
    pub steps: StepsToPerform,
    pub first_token_min_amount_out: BigUint<M>,
    pub second_token_min_amount_out: BigUint<M>,
}

#[multiversx_sc::module]
pub trait CreatePosModule:
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
    fn create_pos_common(&self, args: CreatePosArgs<Self::Api>) -> PaymentsVec<Self::Api> {
        let add_liq_result = self.call_pair_add_liquidity(
            args.dest_pair_address,
            args.pair_input_tokens.first_tokens,
            args.pair_input_tokens.second_tokens,
            args.first_token_min_amount_out,
            args.second_token_min_amount_out,
        );

        let mut output_payments = PaymentsWrapper::new();
        output_payments.push(add_liq_result.first_tokens_remaining);
        output_payments.push(add_liq_result.second_tokens_remaining);

        if matches!(args.steps, StepsToPerform::AddLiquidity) {
            output_payments.push(add_liq_result.lp_tokens);

            return output_payments.send_and_return(&args.caller);
        }

        let opt_enter_result = self.try_enter_farm_with_lp(&add_liq_result.lp_tokens, &args.caller);
        require!(opt_enter_result.is_some(), COULD_NOT_CREATE_POS_ERR_MSG);

        let enter_result = unsafe { opt_enter_result.unwrap_unchecked() };
        output_payments.push(enter_result.rewards);

        if matches!(args.steps, StepsToPerform::EnterFarm) {
            output_payments.push(enter_result.new_farm_token);

            return output_payments.send_and_return(&args.caller);
        }

        let opt_stake_result = self
            .try_enter_metastaking_with_lp_farm_tokens(&enter_result.new_farm_token, &args.caller);
        require!(opt_stake_result.is_some(), COULD_NOT_CREATE_POS_ERR_MSG);

        let stake_result = unsafe { opt_stake_result.unwrap_unchecked() };
        output_payments.push(stake_result.staking_boosted_rewards);
        output_payments.push(stake_result.lp_farm_boosted_rewards);
        output_payments.push(stake_result.dual_yield_tokens);

        output_payments.send_and_return(&args.caller)
    }

    fn buy_half_each_token(
        &self,
        input_tokens: EsdtTokenPayment,
        dest_pair: &ManagedAddress,
    ) -> DoubleSwapResult<Self::Api> {
        require!(input_tokens.token_nonce == 0, "Only fungible ESDT accepted");
        self.require_sc_address(dest_pair);

        let dest_pair_config = self.get_pair_config(dest_pair);
        let tokens_to_pair_mapper = self.pair_address_for_tokens(
            &dest_pair_config.first_token_id,
            &dest_pair_config.second_token_id,
        );
        require!(!tokens_to_pair_mapper.is_empty(), UNKNOWN_PAIR_ERR_MSG);

        let stored_pair_address = tokens_to_pair_mapper.get();
        require!(&stored_pair_address == dest_pair, UNKNOWN_PAIR_ERR_MSG);

        let first_amount = &input_tokens.amount / 2u32;
        let second_amount = &input_tokens.amount - &first_amount;

        let first_swap_tokens = self.perform_tokens_swap(
            input_tokens.token_identifier.clone(),
            first_amount,
            dest_pair_config.first_token_id,
        );
        let second_swap_tokens = self.perform_tokens_swap(
            input_tokens.token_identifier,
            second_amount,
            dest_pair_config.second_token_id,
        );

        DoubleSwapResult {
            first_tokens: first_swap_tokens,
            second_tokens: second_swap_tokens,
        }
    }

    fn balance_token_amounts_through_swaps(
        &self,
        dest_pair_address: ManagedAddress,
        payments: &mut PairTokenPayments<Self::Api>,
    ) {
        let pair_reserves = self.get_pair_reserves(
            &dest_pair_address,
            &payments.first_tokens.token_identifier,
            &payments.second_tokens.token_identifier,
        );
        let first_tokens_price_in_second_token = self.pair_get_equivalent(
            &payments.first_tokens.amount,
            &pair_reserves.first_token_reserves,
            &pair_reserves.second_token_reserves,
        );
        let second_tokens_price_in_first_token = self.pair_get_equivalent(
            &payments.second_tokens.amount,
            &pair_reserves.second_token_reserves,
            &pair_reserves.first_token_reserves,
        );

        let first_token_id = &payments.first_tokens.token_identifier;
        let second_token_id = &payments.second_tokens.token_identifier;
        let (swap_tokens_in, requested_token_id) =
            if payments.second_tokens.amount > first_tokens_price_in_second_token {
                let extra_second_tokens =
                    &payments.second_tokens.amount - &first_tokens_price_in_second_token;
                let swap_amount = extra_second_tokens / 2u32;
                let swap_tokens_in = EsdtTokenPayment::new(second_token_id.clone(), 0, swap_amount);

                (swap_tokens_in, first_token_id.clone())
            } else {
                let extra_first_tokens =
                    &payments.first_tokens.amount - &second_tokens_price_in_first_token;
                let swap_amount = extra_first_tokens / 2u32;
                let swap_tokens_in = EsdtTokenPayment::new(first_token_id.clone(), 0, swap_amount);

                (swap_tokens_in, second_token_id.clone())
            };

        if swap_tokens_in.amount == 0 {
            return;
        }

        let swap_amount = swap_tokens_in.amount.clone();
        let received_tokens =
            self.call_pair_swap(dest_pair_address, swap_tokens_in, requested_token_id);
        if &received_tokens.token_identifier == first_token_id {
            payments.second_tokens.amount -= swap_amount;
            payments.first_tokens.amount += received_tokens.amount;
        } else {
            payments.first_tokens.amount -= swap_amount;
            payments.second_tokens.amount += received_tokens.amount;
        }
    }

    /// mimics the implementation from pair, to not have to do a SC call for this
    fn pair_get_equivalent(
        &self,
        input_token_amount: &BigUint,
        input_token_reserve: &BigUint,
        other_token_reserve: &BigUint,
    ) -> BigUint {
        input_token_amount * other_token_reserve / input_token_reserve
    }

    fn try_enter_farm_with_lp(
        &self,
        lp_tokens: &EsdtTokenPayment,
        user: &ManagedAddress,
    ) -> Option<EnterFarmResultWrapper<Self::Api>> {
        let farm_id_for_lp_tokens = self
            .farm_for_farming_token(&lp_tokens.token_identifier)
            .get();
        if farm_id_for_lp_tokens == NULL_ID {
            return None;
        }

        let farm_address = self.farm_ids().get_address(farm_id_for_lp_tokens)?;
        let enter_result = self.call_enter_farm(farm_address, user.clone(), lp_tokens.clone());

        Some(enter_result)
    }

    fn try_enter_metastaking_with_lp_farm_tokens(
        &self,
        lp_farm_tokens: &EsdtTokenPayment,
        user: &ManagedAddress,
    ) -> Option<StakeProxyResult<Self::Api>> {
        let ms_id_for_tokens = self
            .metastaking_for_lp_farm_token(&lp_farm_tokens.token_identifier)
            .get();
        if ms_id_for_tokens == NULL_ID {
            return None;
        }

        let ms_address = self.metastaking_ids().get_address(ms_id_for_tokens)?;
        let stake_result =
            self.call_metastaking_stake(ms_address, user.clone(), lp_farm_tokens.clone());

        Some(stake_result)
    }
}
