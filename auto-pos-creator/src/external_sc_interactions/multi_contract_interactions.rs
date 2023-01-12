use auto_farm::common::address_to_id_mapper::NULL_ID;
use common_structs::PaymentsVec;

use crate::common::payments_wrapper::PaymentsWraper;

elrond_wasm::imports!();

#[elrond_wasm::module]
pub trait MultiContractInteractionsModule:
    super::pair_actions::PairActionsModule
    + crate::configs::pairs_config::PairsConfigModule
    + utils::UtilsModule
    + auto_farm::whitelists::farms_whitelist::FarmsWhitelistModule
    + auto_farm::whitelists::metastaking_whitelist::MetastakingWhitelistModule
    + auto_farm::external_storage_read::farm_storage_read::FarmStorageReadModule
    + auto_farm::external_storage_read::metastaking_storage_read::MetastakingStorageReadModule
    + crate::configs::auto_farm_config::AutoFarmConfigModule
    + super::farm_actions::FarmActionsModule
    + super::metastaking_actions::MetastakingActionsModule
{
    #[endpoint(createPosFromSingleToken)]
    fn create_pos_from_single_token(
        &self,
        dest_pair_address: ManagedAddress,
    ) -> PaymentsVec<Self::Api> {
        let caller = self.blockchain().get_caller();
        let payment = self.call_value().single_esdt();
        let double_swap_result = self.buy_half_each_token(payment, &dest_pair_address);
        let add_liq_result = self.call_pair_add_liquidity(
            dest_pair_address,
            double_swap_result.first_swap_tokens,
            double_swap_result.second_swap_tokens,
        );

        let mut output_payments = PaymentsWraper::new();
        output_payments.push(add_liq_result.first_tokens_remaining);
        output_payments.push(add_liq_result.second_tokens_remaining);

        let auto_farm_address = self.auto_farm_sc_address().get();
        let opt_farm_tokens =
            self.try_enter_farm_with_lp(&add_liq_result.lp_tokens, &caller, &auto_farm_address);
        if opt_farm_tokens.is_none() {
            output_payments.push(add_liq_result.lp_tokens);

            return output_payments.send_and_return(&caller);
        }

        let farm_tokens = unsafe { opt_farm_tokens.unwrap_unchecked() };
        let opt_ms_tokens = self.try_enter_metastaking_with_lp_farm_tokens(
            &farm_tokens,
            &caller,
            &auto_farm_address,
        );
        if opt_ms_tokens.is_none() {
            output_payments.push(farm_tokens);

            return output_payments.send_and_return(&caller);
        }

        let ms_tokens = unsafe { opt_ms_tokens.unwrap_unchecked() };
        output_payments.push(ms_tokens);

        output_payments.send_and_return(&caller)
    }

    fn try_enter_farm_with_lp(
        &self,
        lp_tokens: &EsdtTokenPayment,
        user: &ManagedAddress,
        auto_farm_address: &ManagedAddress,
    ) -> Option<EsdtTokenPayment> {
        let farm_id_for_lp_tokens = self
            .farm_for_farming_token(&lp_tokens.token_identifier)
            .get_from_address(auto_farm_address);
        if farm_id_for_lp_tokens == NULL_ID {
            return None;
        }

        let farm_address = self
            .farm_ids()
            .get_address_at_address(auto_farm_address, farm_id_for_lp_tokens)?;
        let farm_tokens = self.call_enter_farm(farm_address, user.clone(), lp_tokens.clone());

        Some(farm_tokens)
    }

    fn try_enter_metastaking_with_lp_farm_tokens(
        &self,
        lp_farm_tokens: &EsdtTokenPayment,
        user: &ManagedAddress,
        auto_farm_address: &ManagedAddress,
    ) -> Option<EsdtTokenPayment> {
        let ms_id_for_tokens = self
            .metastaking_for_lp_farm_token(&lp_farm_tokens.token_identifier)
            .get_from_address(auto_farm_address);
        if ms_id_for_tokens == NULL_ID {
            return None;
        }

        let ms_address = self
            .metastaking_ids()
            .get_address_at_address(auto_farm_address, ms_id_for_tokens)?;
        let ms_tokens =
            self.call_metastaking_stake(ms_address, user.clone(), lp_farm_tokens.clone());

        Some(ms_tokens)
    }
}
