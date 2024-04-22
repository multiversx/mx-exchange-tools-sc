use common_structs::PaymentsVec;

use crate::common::payments_wrapper::PaymentsWrapper;

use mergeable::Mergeable;

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[derive(TypeAbi, TopEncode, TopDecode, NestedEncode, NestedDecode, PartialEq, Debug)]
pub struct UnwrapResult<M: ManagedTypeApi> {
    pub farm_tokens: PaymentsWrapper<M>,
    pub rewards: PaymentsWrapper<M>,
}

pub struct ExitMultiFarmResult<M: ManagedTypeApi> {
    pub farming_tokens: PaymentsWrapper<M>,
    pub rewards: PaymentsWrapper<M>,
}

#[derive(TypeAbi, TopEncode, TopDecode, NestedEncode, NestedDecode, PartialEq, Debug)]
pub struct UnwrapAndExitResult<M: ManagedTypeApi> {
    pub farming_tokens: PaymentsWrapper<M>,
    pub rewards: PaymentsWrapper<M>,
}

#[multiversx_sc::module]
pub trait UnwrapFarmTokenModule:
    read_external_storage::ReadExternalStorageModule
    + auto_farm::whitelists::farms_whitelist::FarmsWhitelistModule
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
        unwrap_result.farm_tokens.send_to(&caller);
        unwrap_result.rewards.send_to(&caller);

        unwrap_result
    }

    #[payable("*")]
    #[endpoint(unwrapAndExitFarm)]
    fn unwrap_and_exit_farm(&self) -> UnwrapAndExitResult<Self::Api> {
        let caller = self.blockchain().get_caller();
        let payments = self.get_non_empty_payments();
        let unwrap_result = self.unwrap_common(&caller, payments);
        let mut exit_result = self.exit_all_farms(&caller, unwrap_result.farm_tokens);
        exit_result.rewards.merge_with(unwrap_result.rewards);

        exit_result.farming_tokens.send_to(&caller);
        exit_result.rewards.send_to(&caller);

        UnwrapAndExitResult {
            farming_tokens: exit_result.farming_tokens,
            rewards: exit_result.rewards,
        }
    }

    fn unwrap_common(
        &self,
        caller: &ManagedAddress,
        payments: PaymentsVec<Self::Api>,
    ) -> UnwrapResult<Self::Api> {
        let token_mapper = self.farm_token();
        token_mapper.require_all_same_token(&payments);

        let mut claim_result = self.generate_rewards_all_tokens(caller, payments.clone());
        let unwrap_result = UnwrapResult {
            farm_tokens: claim_result.underlying_farm_tokens,
            rewards: claim_result.rewards,
        };

        let mut total_supply_lost = BigUint::zero();
        for payment in &payments {
            total_supply_lost += payment.amount;
        }

        claim_result.storage_cache.farm_token_supply -= total_supply_lost;

        self.send().esdt_local_burn_multi(&payments);

        unwrap_result
    }

    fn exit_all_farms(
        &self,
        user: &ManagedAddress,
        farm_tokens: PaymentsWrapper<Self::Api>,
    ) -> ExitMultiFarmResult<Self::Api> {
        let mut farming_tokens = PaymentsWrapper::new();
        let mut rewards = PaymentsWrapper::new();
        for farm_token in farm_tokens.iter() {
            let exit_result = self.exit_farm(user.clone(), farm_token);
            farming_tokens.push(exit_result.farming_tokens);
            rewards.push(exit_result.rewards);
        }

        ExitMultiFarmResult {
            farming_tokens,
            rewards,
        }
    }
}
