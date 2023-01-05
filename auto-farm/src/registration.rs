use common_structs::PaymentsVec;

elrond_wasm::imports!();

#[elrond_wasm::module]
pub trait RegistrationModule:
    crate::common::common_storage::CommonStorageModule
    + crate::user_tokens::user_rewards::UserRewardsModule
    + crate::fees::FeesModule
    + crate::external_sc_interactions::locked_token_merging::LockedTokenMergingModule
    + crate::whitelists::farms_whitelist::FarmsWhitelistModule
    + crate::external_sc_interactions::farm_external_storage_read::FarmExternalStorageReadModule
    + crate::user_tokens::user_farm_tokens::UserFarmTokensModule
    + lkmex_transfer::energy_transfer::EnergyTransferModule
    + legacy_token_decode_module::LegacyTokenDecodeModule
    + energy_query::EnergyQueryModule
    + utils::UtilsModule
{
    #[endpoint]
    fn register(&self) {
        let caller = self.blockchain().get_caller();
        let _ = self.user_ids().insert_new(&caller);
    }

    #[endpoint(withdrawAllAndUnregister)]
    fn withdraw_all_and_unregister(&self) -> PaymentsVec<Self::Api> {
        let caller = self.blockchain().get_caller();
        let ids_mapper = self.user_ids();
        let user_id = ids_mapper.get_id_non_zero(&caller);

        let farm_tokens = self.withdraw_farm_tokens(&caller, user_id);
        let claimed_rewards = self.user_claim_rewards(caller, user_id);
        let _ = ids_mapper.remove_by_id(user_id);

        let mut results = farm_tokens;
        results.append_vec(claimed_rewards);

        results
    }
}
