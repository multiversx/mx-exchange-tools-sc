use common_structs::PaymentsVec;

use crate::events::WithdrawType;

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait RegistrationModule:
    crate::common::common_storage::CommonStorageModule
    + crate::user_tokens::user_rewards::UserRewardsModule
    + crate::fees::FeesModule
    + crate::external_sc_interactions::locked_token_merging::LockedTokenMergingModule
    + crate::whitelists::farms_whitelist::FarmsWhitelistModule
    + crate::external_storage_read::farm_storage_read::FarmStorageReadModule
    + crate::user_tokens::user_farm_tokens::UserFarmTokensModule
    + crate::whitelists::metastaking_whitelist::MetastakingWhitelistModule
    + crate::user_tokens::user_metastaking_tokens::UserMetastakingTokensModule
    + crate::external_storage_read::metastaking_storage_read::MetastakingStorageReadModule
    + crate::user_tokens::withdraw_tokens::WithdrawTokensModule
    + crate::events::EventsModule
    + lkmex_transfer::energy_transfer::EnergyTransferModule
    + legacy_token_decode_module::LegacyTokenDecodeModule
    + energy_query::EnergyQueryModule
    + utils::UtilsModule
{
    #[endpoint]
    fn register(&self) {
        let caller = self.blockchain().get_caller();
        let _ = self.user_ids().insert_new(&caller);
        self.emit_user_register_event(&caller);
    }

    #[endpoint(withdrawAllAndUnregister)]
    fn withdraw_all_and_unregister(&self) -> PaymentsVec<Self::Api> {
        let caller = self.blockchain().get_caller();
        let ids_mapper = self.user_ids();
        let user_id = ids_mapper.get_id_non_zero(&caller);

        let farm_tokens = self.withdraw_all_tokens(&caller, &self.user_farm_tokens(user_id));
        let ms_tokens = self.withdraw_all_tokens(&caller, &self.user_metastaking_tokens(user_id));
        let claimed_rewards = self.user_claim_rewards(caller.clone(), user_id);
        let _ = ids_mapper.remove_by_id(user_id);

        let mut results = farm_tokens;
        results.append_vec(ms_tokens);
        results.append_vec(claimed_rewards);

        self.emit_token_withdrawal_event(&caller, WithdrawType::AllTokens, &results);

        results
    }
}
