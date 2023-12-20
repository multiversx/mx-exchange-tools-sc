#![no_std]
#![feature(trait_alias)]

use permissions_module::Permissions;

multiversx_sc::imports!();

pub mod common;
pub mod external_sc_interactions;
pub mod reward_tokens;
pub mod single_token_rewards;
pub mod wrapped_farm_attributes;
pub mod wrapper_actions;

#[multiversx_sc::contract]
pub trait FarmExtraRewardsWrapper:
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
    + crate::wrapper_actions::wrap_farm_token::WrapFarmTokenModule
    + crate::wrapper_actions::generate_rewards::GenerateRewardsModule
    + crate::wrapper_actions::unwrap_farm_token::UnwrapFarmTokenModule
    + crate::external_sc_interactions::farm_interactions::FarmInteractionsModule
{
    #[init]
    fn init(&self, division_safety_constant: BigUint) {
        let caller = self.blockchain().get_caller();
        self.add_permissions(caller, Permissions::OWNER | Permissions::PAUSE);

        require!(division_safety_constant != 0, "Invalid div safety const");
        self.division_safety_constant()
            .set(&division_safety_constant);
    }

    #[upgrade]
    fn upgrade(&self) {}
}
