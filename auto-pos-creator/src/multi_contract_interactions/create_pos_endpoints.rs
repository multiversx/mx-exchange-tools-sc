use common_structs::PaymentsVec;
use farm::EnterFarmResultType;

use crate::{
    common::payments_wrapper::PaymentsWrapper,
    external_sc_interactions::pair_actions::PairTokenPayments,
    multi_contract_interactions::create_pos::COULD_NOT_CREATE_POS_ERR_MSG,
};

use super::create_pos::{CreatePosArgs, StepsToPerform};

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait CreatePosEndpointsModule:
    crate::external_sc_interactions::pair_actions::PairActionsModule
    + crate::configs::pairs_config::PairsConfigModule
    + utils::UtilsModule
    + auto_farm::whitelists::farms_whitelist::FarmsWhitelistModule
    + auto_farm::whitelists::metastaking_whitelist::MetastakingWhitelistModule
    + auto_farm::external_storage_read::farm_storage_read::FarmStorageReadModule
    + auto_farm::external_storage_read::metastaking_storage_read::MetastakingStorageReadModule
    + crate::external_sc_interactions::farm_actions::FarmActionsModule
    + crate::external_sc_interactions::farm_staking_actions::FarmStakingActionsModule
    + crate::external_sc_interactions::metastaking_actions::MetastakingActionsModule
    + crate::external_sc_interactions::egld_wrapper_actions::EgldWrapperActionsModule
    + super::create_pos::CreatePosModule
{
    #[payable("*")]
    #[endpoint(createPosFromSingleToken)]
    fn create_pos_from_single_token(
        &self,
        dest_pair_address: ManagedAddress,
        steps: StepsToPerform,
        add_liq_first_token_min_amount_out: BigUint,
        add_liq_second_token_min_amount_out: BigUint,
    ) -> PaymentsVec<Self::Api> {
        let caller = self.blockchain().get_caller();
        let payment = self.call_value().egld_or_single_esdt();
        let payment_esdt = self.get_esdt_payment(payment);
        let double_swap_result = self.buy_half_each_token(payment_esdt, &dest_pair_address);
        let args = CreatePosArgs {
            caller,
            dest_pair_address,
            pair_input_tokens: double_swap_result,
            steps,
            first_token_min_amount_out: add_liq_first_token_min_amount_out,
            second_token_min_amount_out: add_liq_second_token_min_amount_out,
        };

        self.create_pos_common(args)
    }

    /// Create pos from two payments, entering the pair for the two tokens
    /// It will try doing this with the optimal amounts,
    /// performing swaps before adding liqudity if necessary
    #[payable("*")]
    #[endpoint(createPosFromTwoTokens)]
    fn create_pos_from_two_tokens(
        &self,
        steps: StepsToPerform,
        add_liq_first_token_min_amount_out: BigUint,
        add_liq_second_token_min_amount_out: BigUint,
    ) -> PaymentsVec<Self::Api> {
        let caller = self.blockchain().get_caller();
        let [mut first_payment, mut second_payment] = self.call_value().multi_esdt();
        let wrapped_dest_pair_address = self.get_pair_address_for_tokens(
            &first_payment.token_identifier,
            &second_payment.token_identifier,
        );

        if wrapped_dest_pair_address.is_reverse() {
            core::mem::swap(&mut first_payment, &mut second_payment);
        }

        let dest_pair_address = wrapped_dest_pair_address.unwrap_address();
        let mut pair_input_tokens = PairTokenPayments {
            first_tokens: first_payment,
            second_tokens: second_payment,
        };
        self.balance_token_amounts_through_swaps(dest_pair_address.clone(), &mut pair_input_tokens);

        let args = CreatePosArgs {
            caller,
            dest_pair_address,
            pair_input_tokens,
            steps,
            first_token_min_amount_out: add_liq_first_token_min_amount_out,
            second_token_min_amount_out: add_liq_second_token_min_amount_out,
        };

        self.create_pos_common(args)
    }

    #[payable("*")]
    #[endpoint(createPosFromLp)]
    fn create_pos_from_lp(&self, steps: StepsToPerform) -> PaymentsVec<Self::Api> {
        require!(
            !matches!(steps, StepsToPerform::AddLiquidity),
            "Invalid step"
        );

        let caller = self.blockchain().get_caller();
        let payment = self.call_value().single_esdt();

        let opt_enter_result = self.try_enter_farm_with_lp(&payment, &caller);
        require!(opt_enter_result.is_some(), COULD_NOT_CREATE_POS_ERR_MSG);

        let enter_result = unsafe { opt_enter_result.unwrap_unchecked() };
        let mut output_payments = PaymentsWrapper::new();
        output_payments.push(enter_result.rewards);

        if matches!(steps, StepsToPerform::EnterFarm) {
            output_payments.push(enter_result.new_farm_token);

            return output_payments.send_and_return(&caller);
        }

        let opt_stake_result =
            self.try_enter_metastaking_with_lp_farm_tokens(&enter_result.new_farm_token, &caller);
        require!(opt_stake_result.is_some(), COULD_NOT_CREATE_POS_ERR_MSG);

        let stake_result = unsafe { opt_stake_result.unwrap_unchecked() };
        output_payments.push(stake_result.staking_boosted_rewards);
        output_payments.push(stake_result.lp_farm_boosted_rewards);
        output_payments.push(stake_result.dual_yield_tokens);

        output_payments.send_and_return(&caller)
    }

    #[payable("*")]
    #[endpoint(createFarmStakingPosFromSingleToken)]
    fn create_farm_staking_pos_from_single_token(
        &self,
        farm_staking_address: ManagedAddress,
        min_amount_out: BigUint,
    ) -> EnterFarmResultType<Self::Api> {
        let caller = self.blockchain().get_caller();
        let raw_payments = self.call_value().any_payment();
        let mut payments = match &raw_payments {
            EgldOrMultiEsdtPayment::Egld(egld_amount) => {
                let wegld_payment = self.call_wrap_egld(egld_amount.clone());
                ManagedVec::from_single_item(wegld_payment)
            }
            EgldOrMultiEsdtPayment::MultiEsdt(esdt_payments) => esdt_payments.clone(),
        };

        let farming_token_id = self.get_farm_staking_farming_token_id(&farm_staking_address);
        let first_payment = payments.get(0);
        let (new_farm_token, boosted_rewards_payment) =
            if first_payment.token_identifier == farming_token_id {
                self.call_farm_staking_stake(farm_staking_address, caller.clone(), payments)
                    .into_tuple()
            } else {
                payments.remove(0);

                let wegld_token_id = self.wegld_token_id().get();
                let wegld_token_payment = if first_payment.token_identifier != wegld_token_id {
                    self.perform_tokens_swap(
                        first_payment.token_identifier,
                        first_payment.amount,
                        wegld_token_id,
                    )
                } else {
                    first_payment
                };

                let farming_token_first_payment = self.perform_tokens_swap(
                    wegld_token_payment.token_identifier,
                    wegld_token_payment.amount,
                    farming_token_id,
                );
                let mut farming_token_payments =
                    PaymentsVec::from_single_item(farming_token_first_payment);
                farming_token_payments.append_vec(payments);

                self.call_farm_staking_stake(
                    farm_staking_address,
                    caller.clone(),
                    farming_token_payments,
                )
                .into_tuple()
            };

        require!(new_farm_token.amount >= min_amount_out, "Slippage exceeded");

        self.send()
            .direct_non_zero_esdt_payment(&caller, &new_farm_token);
        self.send()
            .direct_non_zero_esdt_payment(&caller, &boosted_rewards_payment);

        (new_farm_token, boosted_rewards_payment).into()
    }

    fn get_esdt_payment(&self, payment: EgldOrEsdtTokenPayment) -> EsdtTokenPayment {
        if payment.token_identifier.is_egld() {
            self.call_wrap_egld(payment.amount)
        } else {
            payment.unwrap_esdt()
        }
    }
}
