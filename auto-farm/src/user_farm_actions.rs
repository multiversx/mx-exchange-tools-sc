use common_structs::PaymentsVec;
use farm::base_functions::{ClaimRewardsResultType, ClaimRewardsResultWrapper};

use crate::farm_external_storage_read::State;

elrond_wasm::imports!();

#[elrond_wasm::module]
pub trait UserFarmActionsModule:
    crate::common_storage::CommonStorageModule
    + crate::farms_whitelist::FarmsWhitelistModule
    + crate::farm_external_storage_read::FarmExternalStorageReadModule
    + crate::user_farm_tokens::UserFarmTokensModule
    + utils::UtilsModule
{
    /// Arg: user to claim rewards for
    #[endpoint(claimAllFarmRewards)]
    fn claim_all_farm_rewards(&self, user: ManagedAddress) {
        self.require_caller_proxy_claim_address();

        let farms_mapper = self.farm_ids();
        let user_id = self.user_ids().get_id_or_insert(&user);
        let user_tokens_mapper = self.user_farm_tokens(user_id);

        let user_tokens = user_tokens_mapper.get();
        let mut new_user_farm_tokens = PaymentsVec::new();
        for farm_token in &user_tokens {
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

            // TODO: Decide what to do with rewards
            let claim_result = self.call_farm_claim(farm_addr, user.clone(), farm_token);
            new_user_farm_tokens.push(claim_result.new_farm_token);
        }

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
    fn farm_proxy(&self, sc_address: ManagedAddress) -> farm::Proxy<Self::Api>;
}
