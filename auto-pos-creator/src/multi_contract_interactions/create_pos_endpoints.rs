multiversx_sc::imports!();

use common_structs::PaymentsVec;

use crate::{
    common::payments_wrapper::PaymentsWrapper,
    external_sc_interactions::{pair_actions::PairAddLiqArgs, router_actions::SwapOperationType},
};

use super::create_pos::{CreateFarmPosArgs, CreateMetastakingPosArgs};

#[multiversx_sc::module]
pub trait CreatePosEndpointsModule:
    utils::UtilsModule
    + read_external_storage::ReadExternalStorageModule
    + crate::configs::pairs_config::PairsConfigModule
    + crate::external_sc_interactions::pair_actions::PairActionsModule
    + crate::external_sc_interactions::router_actions::RouterActionsModule
    + crate::external_sc_interactions::farm_actions::FarmActionsModule
    + crate::external_sc_interactions::farm_staking_actions::FarmStakingActionsModule
    + crate::external_sc_interactions::metastaking_actions::MetastakingActionsModule
    + crate::external_sc_interactions::egld_wrapper_actions::EgldWrapperActionsModule
    + super::create_pos::CreatePosModule
{
    #[payable("*")]
    #[endpoint(createLpPosFromSingleToken)]
    fn create_lp_pos_from_single_token(
        &self,
        pair_address: ManagedAddress,
        add_liq_first_token_min_amount_out: BigUint,
        add_liq_second_token_min_amount_out: BigUint,
        swap_operations: MultiValueEncoded<SwapOperationType<Self::Api>>,
    ) -> PaymentsVec<Self::Api> {
        let caller = self.blockchain().get_caller();
        let payment = self.call_value().egld_or_single_esdt();

        self.require_sc_address(&pair_address);

        let mut first_token_payment = self.process_payment(payment, swap_operations);
        let second_token_payment =
            self.swap_half_input_payment_if_needed(&mut first_token_payment, pair_address.clone());

        let args = PairAddLiqArgs {
            pair_address,
            first_tokens: first_token_payment,
            second_tokens: second_token_payment,
            first_token_min_amount_out: add_liq_first_token_min_amount_out,
            second_token_min_amount_out: add_liq_second_token_min_amount_out,
        };
        let (new_lp_tokens, mut output_payments) = self.create_lp_pos(args);
        output_payments.push(new_lp_tokens);

        output_payments.send_and_return(&caller)
    }

    #[payable("*")]
    #[endpoint(createLpPosFromTwoTokens)]
    fn create_lp_pos_from_two_tokens(
        &self,
        pair_address: ManagedAddress,
        add_liq_first_token_min_amount_out: BigUint,
        add_liq_second_token_min_amount_out: BigUint,
    ) -> PaymentsVec<Self::Api> {
        let caller = self.blockchain().get_caller();
        self.require_sc_address(&pair_address);

        let [first_token_payment, second_token_payment] = self.call_value().multi_esdt();

        let args = PairAddLiqArgs {
            pair_address,
            first_tokens: first_token_payment,
            second_tokens: second_token_payment,
            first_token_min_amount_out: add_liq_first_token_min_amount_out,
            second_token_min_amount_out: add_liq_second_token_min_amount_out,
        };
        let (new_lp_tokens, mut output_payments) = self.create_lp_pos(args);
        output_payments.push(new_lp_tokens);

        output_payments.send_and_return(&caller)
    }

    #[payable("*")]
    #[endpoint(createFarmPosFromSingleToken)]
    fn create_farm_pos_from_single_token(
        &self,
        farm_address: ManagedAddress,
        add_liq_first_token_min_amount_out: BigUint,
        add_liq_second_token_min_amount_out: BigUint,
        swap_operations: MultiValueEncoded<SwapOperationType<Self::Api>>,
    ) -> PaymentsVec<Self::Api> {
        let caller = self.blockchain().get_caller();
        let (first_payment, additional_payments) = self.split_first_payment();

        let pair_address = self
            .get_farm_pair_contract_address_mapper(farm_address.clone())
            .get();
        self.require_sc_address(&pair_address);

        let mut first_token_payment = self.process_payment(first_payment, swap_operations);
        let second_token_payment =
            self.swap_half_input_payment_if_needed(&mut first_token_payment, pair_address.clone());

        let args = CreateFarmPosArgs {
            caller: caller.clone(),
            first_token_payment,
            second_token_payment,
            additional_payments,
            add_liq_first_token_min_amount_out,
            add_liq_second_token_min_amount_out,
            pair_address,
            farm_address,
        };
        let (new_farm_tokens, mut output_payments) = self.create_farm_pos(args);
        output_payments.push(new_farm_tokens);

        output_payments.send_and_return(&caller)
    }

    #[payable("*")]
    #[endpoint(createFarmPosFromTwoTokens)]
    fn create_farm_pos_from_two_tokens(
        &self,
        farm_address: ManagedAddress,
        add_liq_first_token_min_amount_out: BigUint,
        add_liq_second_token_min_amount_out: BigUint,
    ) -> PaymentsVec<Self::Api> {
        let caller = self.blockchain().get_caller();

        let pair_address = self
            .get_farm_pair_contract_address_mapper(farm_address.clone())
            .get();
        self.require_sc_address(&pair_address);

        let (first_token_payment, second_token_payment, additional_payments) =
            self.split_first_two_payments();

        let args = CreateFarmPosArgs {
            caller: caller.clone(),
            first_token_payment,
            second_token_payment,
            additional_payments,
            add_liq_first_token_min_amount_out,
            add_liq_second_token_min_amount_out,
            pair_address,
            farm_address,
        };
        let (new_farm_tokens, mut output_payments) = self.create_farm_pos(args);
        output_payments.push(new_farm_tokens);

        output_payments.send_and_return(&caller)
    }

    #[payable("*")]
    #[endpoint(createMetastakingPosFromSingleToken)]
    fn create_metastaking_pos_from_single_token(
        &self,
        metastaking_address: ManagedAddress,
        add_liq_first_token_min_amount_out: BigUint,
        add_liq_second_token_min_amount_out: BigUint,
        swap_operations: MultiValueEncoded<SwapOperationType<Self::Api>>,
    ) -> PaymentsVec<Self::Api> {
        let caller = self.blockchain().get_caller();
        let (first_payment, additional_payments) = self.split_first_payment();

        let farm_address = self
            .get_lp_farm_address_mapper(metastaking_address.clone())
            .get();
        let pair_address = self
            .get_farm_pair_contract_address_mapper(farm_address.clone())
            .get();
        self.require_sc_address(&pair_address);

        let mut first_token_payment = self.process_payment(first_payment, swap_operations);
        let second_token_payment =
            self.swap_half_input_payment_if_needed(&mut first_token_payment, pair_address.clone());

        let args = CreateMetastakingPosArgs {
            caller: caller.clone(),
            first_token_payment,
            second_token_payment,
            additional_payments,
            add_liq_first_token_min_amount_out,
            add_liq_second_token_min_amount_out,
            pair_address,
            farm_address,
            metastaking_address,
        };
        let (new_metastaking_tokens, mut output_payments) = self.create_metastaking_pos(args);
        output_payments.push(new_metastaking_tokens);

        output_payments.send_and_return(&caller)
    }

    #[payable("*")]
    #[endpoint(createMetastakingPosFromTwoTokens)]
    fn create_metastaking_pos_from_two_tokens(
        &self,
        metastaking_address: ManagedAddress,
        add_liq_first_token_min_amount_out: BigUint,
        add_liq_second_token_min_amount_out: BigUint,
    ) -> PaymentsVec<Self::Api> {
        let caller = self.blockchain().get_caller();

        let farm_address = self
            .get_lp_farm_address_mapper(metastaking_address.clone())
            .get();
        let pair_address = self
            .get_farm_pair_contract_address_mapper(farm_address.clone())
            .get();
        self.require_sc_address(&pair_address);

        let (first_token_payment, second_token_payment, additional_payments) =
            self.split_first_two_payments();

        let args = CreateMetastakingPosArgs {
            caller: caller.clone(),
            first_token_payment,
            second_token_payment,
            additional_payments,
            add_liq_first_token_min_amount_out,
            add_liq_second_token_min_amount_out,
            pair_address,
            farm_address,
            metastaking_address,
        };
        let (new_metastaking_tokens, mut output_payments) = self.create_metastaking_pos(args);
        output_payments.push(new_metastaking_tokens);

        output_payments.send_and_return(&caller)
    }

    #[payable("*")]
    #[endpoint(createFarmStakingPosFromSingleToken)]
    fn create_farm_staking_pos_from_single_token(
        &self,
        farm_staking_address: ManagedAddress,
        min_amount_out: BigUint,
        swap_operations: MultiValueEncoded<SwapOperationType<Self::Api>>,
    ) -> PaymentsVec<Self::Api> {
        let caller = self.blockchain().get_caller();

        let (first_payment, additional_payments) = self.split_first_payment();

        let token_payment = self.process_payment(first_payment, swap_operations);
        let farming_token_id = self.get_farm_staking_farming_token_id(farm_staking_address.clone());

        require!(
            token_payment.token_identifier == farming_token_id,
            "Invalid swap output token identifier"
        );

        let mut token_payments = PaymentsVec::from_single_item(token_payment);
        token_payments.append_vec(additional_payments);

        let (new_farm_token, boosted_rewards_payment) = self
            .call_farm_staking_stake(farm_staking_address, caller.clone(), token_payments)
            .into_tuple();

        require!(new_farm_token.amount >= min_amount_out, "Slippage exceeded");

        let mut output_payments = PaymentsWrapper::new();
        output_payments.push(boosted_rewards_payment);
        output_payments.push(new_farm_token);

        output_payments.send_and_return(&caller)
    }
}
