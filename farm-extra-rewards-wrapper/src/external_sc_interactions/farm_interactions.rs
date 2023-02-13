use common_structs::PaymentsVec;
use farm::base_functions::{ClaimRewardsResultType, ClaimRewardsResultWrapper};
use unwrappable::Unwrappable;

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait FarmInteractionsModule:
    auto_farm::whitelists::farms_whitelist::FarmsWhitelistModule
    + auto_farm::external_storage_read::farm_storage_read::FarmStorageReadModule
    + utils::UtilsModule
{
    fn claim_base_farm_rewards(
        &self,
        user: ManagedAddress,
        farm_tokens: PaymentsVec<Self::Api>,
    ) -> ClaimRewardsResultWrapper<Self::Api> {
        let first_farm_token = farm_tokens.get(0);
        let farm_id = self
            .farm_for_farm_token(&first_farm_token.token_identifier)
            .get();
        let farm_addr = self
            .farm_ids()
            .get_address(farm_id)
            .unwrap_or_panic::<Self::Api>();

        let raw_results: ClaimRewardsResultType<Self::Api> = self
            .farm_proxy(farm_addr)
            .claim_rewards_endpoint(user)
            .with_multi_token_transfer(farm_tokens)
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
