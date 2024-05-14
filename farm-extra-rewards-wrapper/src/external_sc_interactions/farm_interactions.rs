use common_structs::PaymentsVec;
use farm::{
    base_functions::{ClaimRewardsResultType, ClaimRewardsResultWrapper, ExitFarmResultWrapper},
    ExitFarmWithPartialPosResultType,
};
use unwrappable::Unwrappable;

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait FarmInteractionsModule:
    read_external_storage::ReadExternalStorageModule
    + auto_farm::whitelists::farms_whitelist::FarmsWhitelistModule
    + auto_farm::external_storage_read::farm_storage_read::FarmStorageReadModule
    + utils::UtilsModule
{
    fn claim_base_farm_rewards(
        &self,
        user: ManagedAddress,
        farm_tokens: PaymentsVec<Self::Api>,
    ) -> ClaimRewardsResultWrapper<Self::Api> {
        let first_farm_token = farm_tokens.get(0);
        let farm_addr = self.get_farm_address(&first_farm_token.token_identifier);

        let raw_results: ClaimRewardsResultType<Self::Api> = self
            .farm_proxy(farm_addr)
            .claim_rewards_endpoint(OptionalValue::Some(user))
            .with_multi_token_transfer(farm_tokens)
            .execute_on_dest_context();
        let (new_farm_token, rewards) = raw_results.into_tuple();

        ClaimRewardsResultWrapper {
            new_farm_token,
            rewards,
        }
    }

    fn exit_farm(
        &self,
        user: ManagedAddress,
        farm_token: EsdtTokenPayment,
    ) -> ExitFarmResultWrapper<Self::Api> {
        let farm_addr = self.get_farm_address(&farm_token.token_identifier);
        let raw_results: ExitFarmWithPartialPosResultType<Self::Api> = self
            .farm_proxy(farm_addr)
            .exit_farm_endpoint(OptionalValue::Some(user))
            .with_esdt_transfer(farm_token)
            .execute_on_dest_context();
        let (farming_tokens, rewards) = raw_results.into_tuple();

        ExitFarmResultWrapper {
            farming_tokens,
            rewards,
        }
    }

    fn get_farm_address(&self, farm_token_id: &TokenIdentifier) -> ManagedAddress {
        let farm_id = self.farm_for_farm_token(farm_token_id).get();
        self.farm_ids()
            .get_address(farm_id)
            .unwrap_or_panic::<Self::Api>()
    }

    #[proxy]
    fn farm_proxy(&self, sc_address: ManagedAddress) -> farm_with_locked_rewards::Proxy<Self::Api>;
}
