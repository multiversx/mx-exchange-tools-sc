use metabonding::claim::ClaimArgPair;

use crate::common::rewards_wrapper::RewardsWrapper;

elrond_wasm::imports!();

#[elrond_wasm::module]
pub trait MultiContractInteractionsModule:
    crate::whitelists::farms_whitelist::FarmsWhitelistModule
    + crate::external_storage_read::farm_storage_read::FarmStorageReadModule
    + crate::common::common_storage::CommonStorageModule
    + crate::registration::RegistrationModule
    + crate::user_tokens::user_farm_tokens::UserFarmTokensModule
    + crate::external_sc_interactions::farm_actions::FarmActionsModule
    + crate::external_sc_interactions::metabonding_actions::MetabondingActionsModule
    + crate::external_sc_interactions::fees_collector_actions::FeesCollectorActionsModule
    + crate::external_sc_interactions::locked_token_merging::LockedTokenMergingModule
    + crate::whitelists::metastaking_whitelist::MetastakingWhitelistModule
    + crate::user_tokens::user_metastaking_tokens::UserMetastakingTokensModule
    + crate::external_storage_read::metastaking_storage_read::MetastakingStorageReadModule
    + crate::external_sc_interactions::metastaking_actions::MetastakingActionsModule
    + crate::user_tokens::user_rewards::UserRewardsModule
    + crate::fees::FeesModule
    + lkmex_transfer::energy_transfer::EnergyTransferModule
    + legacy_token_decode_module::LegacyTokenDecodeModule
    + energy_query::EnergyQueryModule
    + utils::UtilsModule
{
    /// Claims rewards from fees collector, metabonding, and farms
    /// Then, compounds rewards into farms where possible
    ///
    /// Args: User to claim for + args required for metabonding claim
    /// Arguments are pairs of:
    /// week: number,
    /// user_delegation_amount: BigUint,
    /// user_lkmex_staked_amount: BigUint,
    /// signature: 120 bytes
    ///
    /// Leave list empty for no metabonding claim
    #[endpoint(claimAllRewardsAndCompound)]
    fn claim_all_rewards_and_compound(
        &self,
        user: ManagedAddress,
        metabonding_claim_args: MultiValueEncoded<ClaimArgPair<Self::Api>>,
    ) {
        self.require_caller_proxy_claim_address();

        let user_id = self.user_ids().get_id_non_zero(&user);
        let locked_token_id = self.get_locked_token_id();
        let mut rew_wrapper = RewardsWrapper::new(locked_token_id);

        self.claim_metabonding_rewards(&user, metabonding_claim_args, &mut rew_wrapper);
        self.claim_fees_collector_rewards(&user, &mut rew_wrapper);
        self.claim_all_farm_rewards(&user, user_id, &mut rew_wrapper);
        self.claim_all_metastaking_rewards(&user, user_id, &mut rew_wrapper);

        self.add_user_rewards(user, user_id, rew_wrapper);
    }
}
