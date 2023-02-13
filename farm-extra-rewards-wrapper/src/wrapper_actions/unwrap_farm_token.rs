use common_structs::PaymentsVec;

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[derive(TypeAbi, TopEncode, TopDecode, NestedEncode, NestedDecode, PartialEq, Debug)]
pub struct UnwrapResult<M: ManagedTypeApi> {
    pub farm_tokens: PaymentsVec<M>,
    pub rewards: PaymentsVec<M>,
}

#[multiversx_sc::module]
pub trait UnwrapFarmTokenModule:
    auto_farm::whitelists::farms_whitelist::FarmsWhitelistModule
    + auto_farm::external_storage_read::farm_storage_read::FarmStorageReadModule
    + rewards::RewardsModule
    + config::ConfigModule
    + token_send::TokenSendModule
    + farm_token::FarmTokenModule
    + pausable::PausableModule
    + permissions_module::PermissionsModule
    + multiversx_sc_modules::default_issue_callbacks::DefaultIssueCallbacksModule
    + farm_base_impl::base_farm_validation::BaseFarmValidationModule
    + farm_base_impl::enter_farm::BaseEnterFarmModule
    + utils::UtilsModule
    + crate::reward_tokens::RewardTokensModule
    + crate::external_sc_interactions::farm_interactions::FarmInteractionsModule
    + super::generate_rewards::GenerateRewardsModule
{
    #[payable("*")]
    #[endpoint(unwrapFarmToken)]
    fn unwrap_farm_token(&self) -> UnwrapResult<Self::Api> {
        let caller = self.blockchain().get_caller();
        let payments = self.get_non_empty_payments();
        let unwrap_result = self.unwrap_common(&caller, payments);
        if !unwrap_result.farm_tokens.is_empty() {
            self.send()
                .direct_multi(&caller, &unwrap_result.farm_tokens);
        }
        if !unwrap_result.rewards.is_empty() {
            self.send().direct_multi(&caller, &unwrap_result.rewards);
        }

        unwrap_result
    }

    fn unwrap_common(
        &self,
        caller: &ManagedAddress,
        payments: PaymentsVec<Self::Api>,
    ) -> UnwrapResult<Self::Api> {
        let token_mapper = self.farm_token();
        token_mapper.require_all_same_token(&payments);

        let claim_result = self.generate_rewards_all_tokens(caller, payments.clone());
        let unwrap_result = UnwrapResult {
            farm_tokens: claim_result.underlying_farm_tokens,
            rewards: claim_result.rewards,
        };

        self.send().esdt_local_burn_multi(&payments);

        unwrap_result
    }
}
