use auto_farm::common::address_to_id_mapper::NULL_ID;
use common_structs::PaymentsVec;

use crate::common::payments_wrapper::PaymentsWrapper;

elrond_wasm::imports!();

pub enum ExitType<M: ManagedTypeApi> {
    Metastaking(ManagedAddress<M>),
    Farm(ManagedAddress<M>),
    Pair(ManagedAddress<M>),
}

static INVALID_INPUT_TOKEN_ERR_MSG: &[u8] = b"Invalid input token";

#[elrond_wasm::module]
pub trait ExitPosModule:
    crate::external_sc_interactions::pair_actions::PairActionsModule
    + crate::configs::pairs_config::PairsConfigModule
    + utils::UtilsModule
    + auto_farm::whitelists::farms_whitelist::FarmsWhitelistModule
    + auto_farm::whitelists::metastaking_whitelist::MetastakingWhitelistModule
    + auto_farm::external_storage_read::farm_storage_read::FarmStorageReadModule
    + auto_farm::external_storage_read::metastaking_storage_read::MetastakingStorageReadModule
    + crate::configs::auto_farm_config::AutoFarmConfigModule
    + crate::external_sc_interactions::farm_actions::FarmActionsModule
    + crate::external_sc_interactions::metastaking_actions::MetastakingActionsModule
{
    #[payable("*")]
    #[endpoint(fullExitPos)]
    fn full_exit_pos(&self) -> PaymentsVec<Self::Api> {
        let caller = self.blockchain().get_caller();
        let payment = self.call_value().single_esdt();

        let auto_farm_sc_address = self.auto_farm_sc_address().get();
        let exit_type = self.get_exit_type(&payment.token_identifier, &auto_farm_sc_address);
        let mut output_payments = PaymentsWrapper::new();

        match exit_type {
            ExitType::Metastaking(ms_addr) => {
                self.unstake_metastaking(&mut output_payments, ms_addr, caller.clone(), payment)
            }
            ExitType::Farm(farm_addr) => {
                self.exit_farm(&mut output_payments, farm_addr, caller.clone(), payment)
            }
            ExitType::Pair(pair_addr) => {
                self.remove_pair_liq(&mut output_payments, pair_addr, payment)
            }
        };

        output_payments.send_and_return(&caller)
    }

    fn get_exit_type(
        &self,
        input_token: &TokenIdentifier,
        auto_farm_address: &ManagedAddress,
    ) -> ExitType<Self::Api> {
        let ms_id = self
            .metastaking_for_dual_yield_token(input_token)
            .get_from_address(auto_farm_address);
        if ms_id != NULL_ID {
            let opt_ms_addr = self
                .metastaking_ids()
                .get_address_at_address(auto_farm_address, ms_id);
            require!(opt_ms_addr.is_some(), INVALID_INPUT_TOKEN_ERR_MSG);

            let ms_addr = unsafe { opt_ms_addr.unwrap_unchecked() };
            return ExitType::Metastaking(ms_addr);
        }

        let farm_id = self
            .farm_for_farm_token(input_token)
            .get_from_address(auto_farm_address);
        if farm_id != NULL_ID {
            let opt_farm_addr = self
                .farm_ids()
                .get_address_at_address(auto_farm_address, farm_id);
            require!(opt_farm_addr.is_some(), INVALID_INPUT_TOKEN_ERR_MSG);

            let farm_addr = unsafe { opt_farm_addr.unwrap_unchecked() };
            return ExitType::Farm(farm_addr);
        }

        let pair_addr_mapper = self.pair_for_lp_token(input_token);
        require!(!pair_addr_mapper.is_empty(), INVALID_INPUT_TOKEN_ERR_MSG);

        ExitType::Pair(pair_addr_mapper.get())
    }

    fn unstake_metastaking(
        &self,
        output_payments: &mut PaymentsWrapper<Self::Api>,
        ms_address: ManagedAddress,
        user: ManagedAddress,
        ms_tokens: EsdtTokenPayment,
    ) {
        let unstake_result = self.call_metastaking_unstake(ms_address, user, ms_tokens);
        output_payments.push(unstake_result.other_token_payment);
        output_payments.push(unstake_result.lp_farm_rewards);
        output_payments.push(unstake_result.staking_rewards);
        output_payments.push(unstake_result.unbond_staking_farm_token);

        if let Some(new_dual_yield_tokens) = unstake_result.opt_new_dual_yield_tokens {
            output_payments.push(new_dual_yield_tokens);
        }
    }

    fn exit_farm(
        &self,
        output_payments: &mut PaymentsWrapper<Self::Api>,
        farm_address: ManagedAddress,
        user: ManagedAddress,
        farm_tokens: EsdtTokenPayment,
    ) {
        let exit_farm_result = self.call_exit_farm(farm_address, user, farm_tokens);
        output_payments.push(exit_farm_result.rewards);

        let lp_tokens = exit_farm_result.farming_tokens;
        let pair_addr_mapper = self.pair_for_lp_token(&lp_tokens.token_identifier);
        if pair_addr_mapper.is_empty() {
            output_payments.push(lp_tokens);

            return;
        }

        let pair_addr = pair_addr_mapper.get();
        self.remove_pair_liq(output_payments, pair_addr, lp_tokens);
    }

    fn remove_pair_liq(
        &self,
        output_payments: &mut PaymentsWrapper<Self::Api>,
        pair_address: ManagedAddress,
        lp_tokens: EsdtTokenPayment,
    ) {
        let remove_liq_result = self.call_pair_remove_liquidity(pair_address, lp_tokens);
        output_payments.push(remove_liq_result.first_tokens);
        output_payments.push(remove_liq_result.second_tokens);
    }
}
