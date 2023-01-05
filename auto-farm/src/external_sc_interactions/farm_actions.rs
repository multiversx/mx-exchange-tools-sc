use common_structs::PaymentsVec;
use farm::base_functions::{ClaimRewardsResultType, ClaimRewardsResultWrapper};

use crate::{
    common::unique_payments::UniquePayments,
    external_sc_interactions::farm_external_storage_read::State,
};

elrond_wasm::imports!();

#[elrond_wasm::module]
pub trait FarmActionsModule:
    crate::common::common_storage::CommonStorageModule
    + crate::whitelists::farms_whitelist::FarmsWhitelistModule
    + crate::external_sc_interactions::farm_external_storage_read::FarmExternalStorageReadModule
    + crate::user_tokens::user_farm_tokens::UserFarmTokensModule
    + crate::user_tokens::user_rewards::UserRewardsModule
    + crate::fees::FeesModule
    + utils::UtilsModule
    + energy_query::EnergyQueryModule
    + crate::external_sc_interactions::locked_token_merging::LockedTokenMergingModule
    + lkmex_transfer::energy_transfer::EnergyTransferModule
    + legacy_token_decode_module::LegacyTokenDecodeModule
{
    /// Arg: user to claim rewards for
    #[endpoint(claimAllFarmRewards)]
    fn claim_all_farm_rewards(&self, user: ManagedAddress) {
        self.require_caller_proxy_claim_address();

        let farms_mapper = self.farm_ids();
        let user_id = self.user_ids().get_id_non_zero(&user);

        let locked_token_id = self.get_locked_token_id();
        let user_tokens_mapper = self.user_farm_tokens(user_id);
        let user_farm_tokens = user_tokens_mapper.get();

        let mut new_user_farm_tokens = PaymentsVec::new();
        let mut locked_rewards = UniquePayments::new();
        let mut other_token_rewards = UniquePayments::new();
        for farm_token in &user_farm_tokens {
            let farm_id = self.farm_for_farm_token(&farm_token.token_identifier).get();
            let opt_farm_addr = farms_mapper.get_address(farm_id);
            if opt_farm_addr.is_none() {
                new_user_farm_tokens.push(farm_token);
                continue;
            }

            let farm_addr = unsafe { opt_farm_addr.unwrap_unchecked() };
            let farm_state = self.get_farm_state(&farm_addr);
            if farm_state != State::Active {
                new_user_farm_tokens.push(farm_token);
                continue;
            }

            let claim_result = self.call_farm_claim(farm_addr, user.clone(), farm_token);
            new_user_farm_tokens.push(claim_result.new_farm_token);

            if claim_result.rewards.token_identifier == locked_token_id {
                locked_rewards.add_payment(claim_result.rewards);
            } else {
                other_token_rewards.add_payment(claim_result.rewards);
            }
        }

        self.add_user_rewards(user, user_id, locked_rewards, other_token_rewards);

        user_tokens_mapper.set(&new_user_farm_tokens);
    }

    fn call_farm_claim(
        &self,
        farm_addr: ManagedAddress,
        user: ManagedAddress,
        farm_token: EsdtTokenPayment,
    ) -> ClaimRewardsResultWrapper<Self::Api> {
        let raw_results: ClaimRewardsResultType<Self::Api> = self
            .farm_proxy(farm_addr)
            .claim_rewards_endpoint(user)
            .with_esdt_transfer(farm_token)
            .execute_on_dest_context();
        let (new_farm_token, rewards) = raw_results.into_tuple();

        ClaimRewardsResultWrapper {
            new_farm_token,
            rewards,
        }
    }

    #[proxy]
    fn farm_proxy(&self, sc_address: ManagedAddress) -> farm_with_locked_rewards::Proxy<Self::Api>;
}
