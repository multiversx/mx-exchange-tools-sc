use simple_lock::{
    proxy_farm::{FarmType, ProxyTrait as _},
    proxy_lp::ProxyTrait as _,
};

use crate::action_type::DeployActionType;

use super::common::GAS_AFTER_ASYNC;

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait SimpleLockModule:
    crate::fee::FeeModule
    + super::common::CommonModule
    + utils::UtilsModule
    + multiversx_sc_modules::pause::PauseModule
{
    #[only_owner]
    #[endpoint(setSimpleLockSourceAddress)]
    fn set_simple_lock_source_address(&self, simple_lock_source: ManagedAddress) {
        self.require_sc_address(&simple_lock_source);

        self.simple_lock_source().set(simple_lock_source);
    }

    #[payable("*")]
    #[endpoint(deploySimpleLock)]
    fn deploy_simple_lock(&self) {
        self.require_not_paused();

        let caller = self.blockchain().get_caller();
        let payment = self.call_value().single_esdt();
        self.take_fee(&caller, payment, DeployActionType::SimpleLock);

        let simple_lock_source = self.simple_lock_source().get();
        let code_metadata = self.get_default_code_metadata();
        let (deployed_sc_address, ()) = self
            .simple_lock_proxy()
            .init()
            .deploy_from_source(&simple_lock_source, code_metadata);

        let _ = self.deployed_contracts(&caller).insert(deployed_sc_address);
    }

    #[payable("EGLD")]
    #[endpoint(simpleLockIssueFarmProxyToken)]
    fn simple_lock_issue_farm_proxy_token(
        &self,
        simple_lock_address: ManagedAddress,
        token_display_name: ManagedBuffer,
        token_ticker: ManagedBuffer,
        num_decimals: usize,
    ) {
        self.require_not_paused();

        let caller = self.blockchain().get_caller();
        self.require_deployed_contract(&caller, &simple_lock_address);

        let egld_value = self.call_value().egld_value().clone_value();
        let gas_left = self.blockchain().get_gas_left();
        let _: () = self
            .simple_lock_proxy()
            .contract(simple_lock_address)
            .issue_farm_proxy_token(token_display_name, token_ticker, num_decimals)
            .with_gas_limit(gas_left - GAS_AFTER_ASYNC)
            .with_egld_transfer(egld_value)
            .execute_on_dest_context();
    }

    /// Add a farm to the whitelist.
    /// Currently, two types of farms are supported, denoted by the `farm_type` argument:
    /// `0` - SimpleFarm - rewards are fungible tokens
    /// `1` - FarmWithLockedRewards - rewards are META ESDT locked tokens
    #[endpoint(simpleLockAddFarmToWhitelist)]
    fn simple_lock_add_farm_to_whitelist(
        &self,
        simple_lock_address: ManagedAddress,
        farm_address: ManagedAddress,
        farming_token_id: TokenIdentifier,
        farm_type: FarmType,
    ) {
        self.require_not_paused();

        let caller = self.blockchain().get_caller();
        self.require_deployed_contract(&caller, &simple_lock_address);

        let _: () = self
            .simple_lock_proxy()
            .contract(simple_lock_address)
            .add_farm_to_whitelist(farm_address, farming_token_id, farm_type)
            .execute_on_dest_context();
    }

    #[endpoint(simpleLockRemoveFarmFromWhitelist)]
    fn simple_lock_remove_farm_from_whitelist(
        &self,
        simple_lock_address: ManagedAddress,
        farm_address: ManagedAddress,
        farming_token_id: TokenIdentifier,
        farm_type: FarmType,
    ) {
        self.require_not_paused();

        let caller = self.blockchain().get_caller();
        self.require_deployed_contract(&caller, &simple_lock_address);

        let _: () = self
            .simple_lock_proxy()
            .contract(simple_lock_address)
            .remove_farm_from_whitelist(farm_address, farming_token_id, farm_type)
            .execute_on_dest_context();
    }

    #[payable("EGLD")]
    #[endpoint(simpleLockIssueLpProxyToken)]
    fn simple_lock_issue_lp_proxy_token(
        &self,
        simple_lock_address: ManagedAddress,
        token_display_name: ManagedBuffer,
        token_ticker: ManagedBuffer,
        num_decimals: usize,
    ) {
        self.require_not_paused();

        let caller = self.blockchain().get_caller();
        self.require_deployed_contract(&caller, &simple_lock_address);

        let egld_value = self.call_value().egld_value().clone_value();
        let gas_left = self.blockchain().get_gas_left();
        let _: () = self
            .simple_lock_proxy()
            .contract(simple_lock_address)
            .issue_lp_proxy_token(token_display_name, token_ticker, num_decimals)
            .with_gas_limit(gas_left - GAS_AFTER_ASYNC)
            .with_egld_transfer(egld_value)
            .execute_on_dest_context();
    }

    /// Add a liquidity pool to the whitelist.
    /// If the token pair does not have an associated pool, users may not add liquidity.
    ///
    /// `first_token_id` and `second_token_id` MUST match the LP's order,
    /// otherwise all attempts at adding liquidity will fail
    ///
    /// May not add pools for both pairs, i.e. (first, second) and (second, first)
    #[endpoint(simpleLockAddLpToWhitelist)]
    fn simple_lock_add_lp_to_whitelist(
        &self,
        simple_lock_address: ManagedAddress,
        lp_address: ManagedAddress,
        first_token_id: TokenIdentifier,
        second_token_id: TokenIdentifier,
    ) {
        self.require_not_paused();

        let caller = self.blockchain().get_caller();
        self.require_deployed_contract(&caller, &simple_lock_address);

        let _: () = self
            .simple_lock_proxy()
            .contract(simple_lock_address)
            .add_lp_to_whitelist(lp_address, first_token_id, second_token_id)
            .execute_on_dest_context();
    }

    #[endpoint(simpleLockRemoveLpFromWhitelist)]
    fn simple_lock_remove_lp_from_whitelist(
        &self,
        simple_lock_address: ManagedAddress,
        lp_address: ManagedAddress,
        first_token_id: TokenIdentifier,
        second_token_id: TokenIdentifier,
    ) {
        self.require_not_paused();

        let caller = self.blockchain().get_caller();
        self.require_deployed_contract(&caller, &simple_lock_address);

        let _: () = self
            .simple_lock_proxy()
            .contract(simple_lock_address)
            .remove_lp_from_whitelist(lp_address, first_token_id, second_token_id)
            .execute_on_dest_context();
    }

    #[proxy]
    fn simple_lock_proxy(&self) -> simple_lock::Proxy<Self::Api>;

    #[storage_mapper("simpleLockSource")]
    fn simple_lock_source(&self) -> SingleValueMapper<ManagedAddress>;
}
