multiversx_sc::imports!();

use common_structs::PaymentsVec;

use crate::common::errors::{ERROR_BAD_PAYMENT_TOKENS, ERROR_LOCKED_TOKENS_NOT_FOUND};

#[multiversx_sc::module]
pub trait LockedTokenInteractionsModule:
    read_external_storage::ReadExternalStorageModule
    + crate::external_sc_interactions::farm_actions::FarmActionsModule
    + crate::external_sc_interactions::energy_dao_config::EnergyDAOConfigModule
    + crate::external_sc_interactions::locked_token_actions::LockedTokenModule
    + utils::UtilsModule
    + permissions_module::PermissionsModule
    + energy_query::EnergyQueryModule
    + lkmex_transfer::energy_transfer::EnergyTransferModule
    + legacy_token_decode_module::LegacyTokenDecodeModule
    + crate::wrapped_token::WrappedTokenModule
    + simple_lock::token_attributes::TokenAttributesModule
    + multiversx_sc_modules::default_issue_callbacks::DefaultIssueCallbacksModule
{
    /// Endpoint that allows the owner to lock tokens in order to get energy for the contract
    /// It can be further extended to receive tokens from multiple whitelisted addresses, or to merge all locked tokens
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

    /// Endpoint that can extend the locking period of all the locked tokens to a specific epoch
    /// It can also receive an optional nonce argument, to only extend the lock period of a single token
    #[only_owner]
    #[endpoint(extendLockPeriod)]
    fn extend_lock_period(&self, lock_epoch: u64, opt_nonce_to_update: OptionalValue<u64>) {
        let locked_tokens_mapper = self.internal_locked_tokens();
        require!(
            !locked_tokens_mapper.is_empty(),
            ERROR_LOCKED_TOKENS_NOT_FOUND
        );
        let initial_locked_tokens = locked_tokens_mapper.get();
        let nonce_to_update = match opt_nonce_to_update {
            OptionalValue::Some(nonce_to_update) => nonce_to_update,
            OptionalValue::None => 0u64,
        };

        let mut new_locked_tokens = ManagedVec::new();
        for locked_token in initial_locked_tokens.iter() {
            if locked_token.token_nonce == nonce_to_update || nonce_to_update == 0u64 {
                let new_token = self.lock_tokens(locked_token, lock_epoch);
                new_locked_tokens.push(new_token);
            } else {
                new_locked_tokens.push(locked_token);
            }
        }

        locked_tokens_mapper.set(new_locked_tokens);
    }

    #[view(getInternalLockedTokens)]
    #[storage_mapper("internalLockedTokens")]
    fn internal_locked_tokens(&self) -> SingleValueMapper<PaymentsVec<Self::Api>>;
}
