#![no_std]

multiversx_sc::imports!();

pub mod single_token_rewards;
pub mod reward_tokens;
pub mod wrapped_farm_attributes;

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
    + utils::UtilsModule
    + crate::reward_tokens::RewardTokensModule
{
    #[init]
    fn init(&self) {}
}
