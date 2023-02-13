use crate::single_token_rewards::BaseFarmLogicWrapper;

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait WrapFarmTokenModule:
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
{
    /// Wraps a single farm token
    #[payable("*")]
    #[endpoint(wrapFarmToken)]
    fn wrap_farm_token_endpoint(&self) -> EsdtTokenPayment {
        let farm_token = self.call_value().single_esdt();
        let farm_id = self.farm_for_farm_token(&farm_token.token_identifier).get();
        let opt_farm_address = self.farm_ids().get_address(farm_id);
        require!(opt_farm_address.is_some(), "Invalid farm token");

        // To pass the `validate_contract_state` checks in `enter_farm_base`
        self.overwrite_farming_token(&farm_token.token_identifier);

        let caller = self.blockchain().get_caller();
        let enter_result = self.enter_farm_base::<BaseFarmLogicWrapper<Self>>(
            caller.clone(),
            ManagedVec::from_single_item(farm_token),
        );

        let wrapped_token = enter_result.new_farm_token.payment;
        self.send()
            .direct_non_zero_esdt_payment(&caller, &wrapped_token);

        wrapped_token
    }

    fn overwrite_farming_token(&self, new_token: &TokenIdentifier) {
        <Self as config::ConfigModule>::farming_token_id(self).set(new_token);
    }
}
