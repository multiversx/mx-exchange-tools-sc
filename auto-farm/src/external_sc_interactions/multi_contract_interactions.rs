multiversx_sc::imports!();

use crate::common::{rewards_wrapper::RewardsWrapper, unique_payments::UniquePayments};

#[multiversx_sc::module]
pub trait MultiContractInteractionsModule:
    read_external_storage::ReadExternalStorageModule
    + crate::whitelists::farms_whitelist::FarmsWhitelistModule
    + crate::external_storage_read::farm_storage_read::FarmStorageReadModule
    + crate::common::common_storage::CommonStorageModule
    + crate::registration::RegistrationModule
    + crate::user_tokens::user_farm_tokens::UserFarmTokensModule
    + crate::external_sc_interactions::farm_actions::FarmActionsModule
    + crate::external_sc_interactions::fees_collector_actions::FeesCollectorActionsModule
    + crate::external_sc_interactions::locked_token_merging::LockedTokenMergingModule
    + crate::whitelists::metastaking_whitelist::MetastakingWhitelistModule
    + crate::user_tokens::user_metastaking_tokens::UserMetastakingTokensModule
    + crate::external_storage_read::metastaking_storage_read::MetastakingStorageReadModule
    + crate::external_sc_interactions::metastaking_actions::MetastakingActionsModule
    + crate::user_tokens::user_rewards::UserRewardsModule
    + crate::user_tokens::withdraw_tokens::WithdrawTokensModule
    + crate::fees::FeesModule
    + crate::events::EventsModule
    + lkmex_transfer::energy_transfer::EnergyTransferModule
    + legacy_token_decode_module::LegacyTokenDecodeModule
    + energy_query::EnergyQueryModule
    + utils::UtilsModule
{
    /// Claims rewards from fees collector, and farms
    /// Then, compounds rewards into farms where possible
    ///
    /// Args: Pairs of user to claim for
    #[endpoint(claimAllRewardsAndCompound)]
    fn claim_all_rewards_and_compound(&self, users: MultiValueEncoded<ManagedAddress>) {
        self.require_caller_proxy_claim_address();

        let locked_token_id = self.get_locked_token_id();
        for user in users {
            self.claim_all_single_user(&user, locked_token_id.clone());
        }
    }

    fn claim_all_single_user(&self, user: &ManagedAddress, locked_token_id: TokenIdentifier) {
        let user_id = self.user_ids().get_id_non_zero(user);
        let mut rew_wrapper = RewardsWrapper::new(locked_token_id);

        self.claim_fees_collector_rewards(user, &mut rew_wrapper);
        self.claim_all_farm_rewards(user, user_id, &mut rew_wrapper);
        self.claim_all_metastaking_rewards(user, user_id, &mut rew_wrapper);

        self.add_user_rewards(user.clone(), user_id, rew_wrapper);

        let user_rewards_mapper = self.user_rewards(user_id);
        let user_farm_tokens_mapper = self.user_farm_tokens(user_id);

        let mut user_wrapped_rewards = user_rewards_mapper.get();
        let mut user_rewards = user_wrapped_rewards.other_tokens.into_payments();
        let mut user_farm_tokens = user_farm_tokens_mapper.get();
        let user_farm_ids = self.get_farm_ids_for_farm_tokens(&user_farm_tokens);

        let mut i = 0;
        let mut len = user_rewards.len();
        while i < len {
            let current_payment = user_rewards.get(i);
            let compound_result = self.compound_staking_rewards_with_existing_farm_position(
                user,
                &mut user_farm_tokens,
                &user_farm_ids,
                current_payment,
            );
            if compound_result.is_err() {
                i += 1;
                continue;
            }

            user_rewards.remove(i);
            len -= 1;
        }

        user_wrapped_rewards.other_tokens = UniquePayments::new_from_unique_payments(user_rewards);
        user_rewards_mapper.set(&user_wrapped_rewards);
        user_farm_tokens_mapper.set(&user_farm_tokens);

        self.emit_proxy_claim_event(user, &user_wrapped_rewards, &user_farm_tokens);
    }
}
