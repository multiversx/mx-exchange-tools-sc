multiversx_sc::imports!();

use common_structs::PaymentsVec;

use crate::common::errors::ERROR_BAD_PAYMENT_TOKENS;

#[multiversx_sc::module]
pub trait LockedTokenInteractionsModule:
    crate::external_sc_interactions::farm_actions::FarmActionsModule
    + crate::external_sc_interactions::farm_config::FarmConfigModule
    + crate::external_sc_interactions::locked_token_actions::LockedTokenModule
    + utils::UtilsModule
    + energy_query::EnergyQueryModule
    + lkmex_transfer::energy_transfer::EnergyTransferModule
    + legacy_token_decode_module::LegacyTokenDecodeModule
    + crate::wrapped_token::WrappedTokenModule
    + simple_lock::token_attributes::TokenAttributesModule
    + multiversx_sc_modules::default_issue_callbacks::DefaultIssueCallbacksModule
{
    // Owner can lock tokens in order to get energy for the contract
    #[only_owner]
    #[payable("*")]
    #[endpoint(lockEnergyTokens)]
    fn lock_energy_tokens(&self, lock_epoch: u64) {
        let payment = self.call_value().single_esdt();
        let base_token_id = self.get_base_token_id();
        require!(
            payment.token_identifier == base_token_id,
            ERROR_BAD_PAYMENT_TOKENS
        );

        let new_locked_tokens = self.lock_tokens(payment, lock_epoch);
        self.internal_locked_tokens()
            .update(|locked_tokens| locked_tokens.push(new_locked_tokens));
    }

    #[storage_mapper("internalLockedTokens")]
    fn internal_locked_tokens(&self) -> SingleValueMapper<PaymentsVec<Self::Api>>;
}
