use proxy_dex::other_sc_whitelist::ProxyTrait as _;

use crate::action_type::DeployActionType;

use super::common::GAS_AFTER_ASYNC;

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait ProxyDexModule:
    crate::fee::FeeModule
    + super::common::CommonModule
    + utils::UtilsModule
    + multiversx_sc_modules::pause::PauseModule
{
    #[only_owner]
    #[endpoint(setProxyDexSourceAddress)]
    fn set_proxy_dex_source_address(&self, proxy_dex_source: ManagedAddress) {
        self.require_sc_address(&proxy_dex_source);

        self.proxy_dex_source().set(proxy_dex_source);
    }

    #[payable("*")]
    #[endpoint(deployProxyDex)]
    fn deploy_proxy_dex(
        &self,
        old_locked_token_id: TokenIdentifier,
        old_factory_address: ManagedAddress,
        energy_factory_address: ManagedAddress,
    ) {
        self.require_not_paused();

        let caller = self.blockchain().get_caller();
        let payment = self.call_value().single_esdt();
        self.take_fee(&caller, payment, DeployActionType::ProxyDex);

        let proxy_dex_source = self.proxy_dex_source().get();
        let code_metadata = self.get_default_code_metadata();
        let (deployed_sc_address, ()) = self
            .proxy_dex_proxy()
            .init(
                old_locked_token_id,
                old_factory_address,
                energy_factory_address,
            )
            .deploy_from_source(&proxy_dex_source, code_metadata);

        let _ = self.deployed_contracts(&caller).insert(deployed_sc_address);
    }

    #[payable("EGLD")]
    #[endpoint(proxyDexRegisterProxyPair)]
    fn proxy_dex_register_proxy_pair(
        &self,
        proxy_dex_address: ManagedAddress,
        token_display_name: ManagedBuffer,
        token_ticker: ManagedBuffer,
        num_decimals: usize,
    ) {
        self.require_not_paused();

        let caller = self.blockchain().get_caller();
        self.require_deployed_contract(&caller, &proxy_dex_address);

        let egld_value = self.call_value().egld_value().clone_value();
        let gas_left = self.blockchain().get_gas_left();
        let _: () = self
            .proxy_dex_proxy()
            .contract(proxy_dex_address)
            .register_proxy_pair(token_display_name, token_ticker, num_decimals)
            .with_gas_limit(gas_left - GAS_AFTER_ASYNC)
            .with_egld_transfer(egld_value)
            .execute_on_dest_context();
    }

    #[endpoint(proxyDexSetTransferRoleWrappedLpToken)]
    fn proxy_dex_set_transfer_role_wrapped_lp_token(&self, proxy_dex_address: ManagedAddress) {
        self.require_not_paused();

        let caller = self.blockchain().get_caller();
        self.require_deployed_contract(&caller, &proxy_dex_address);

        let gas_left = self.blockchain().get_gas_left();
        let _: () = self
            .proxy_dex_proxy()
            .contract(proxy_dex_address)
            .set_transfer_role_wrapped_lp_token(OptionalValue::<ManagedAddress>::None)
            .with_gas_limit(gas_left - GAS_AFTER_ASYNC)
            .execute_on_dest_context();
    }

    #[payable("EGLD")]
    #[endpoint(proxyDexRegisterProxyFarm)]
    fn proxy_dex_register_proxy_farm(
        &self,
        proxy_dex_address: ManagedAddress,
        token_display_name: ManagedBuffer,
        token_ticker: ManagedBuffer,
        num_decimals: usize,
    ) {
        self.require_not_paused();

        let caller = self.blockchain().get_caller();
        self.require_deployed_contract(&caller, &proxy_dex_address);

        let egld_value = self.call_value().egld_value().clone_value();
        let gas_left = self.blockchain().get_gas_left();
        let _: () = self
            .proxy_dex_proxy()
            .contract(proxy_dex_address)
            .register_proxy_farm(token_display_name, token_ticker, num_decimals)
            .with_gas_limit(gas_left - GAS_AFTER_ASYNC)
            .with_egld_transfer(egld_value)
            .execute_on_dest_context();
    }

    #[endpoint(proxyDexSetTransferRoleWrappedFarmToken)]
    fn proxy_dex_set_transfer_role_wrapped_farm_token(&self, proxy_dex_address: ManagedAddress) {
        self.require_not_paused();

        let caller = self.blockchain().get_caller();
        self.require_deployed_contract(&caller, &proxy_dex_address);

        let gas_left = self.blockchain().get_gas_left();
        let _: () = self
            .proxy_dex_proxy()
            .contract(proxy_dex_address)
            .set_transfer_role_wrapped_farm_token(OptionalValue::<ManagedAddress>::None)
            .with_gas_limit(gas_left - GAS_AFTER_ASYNC)
            .execute_on_dest_context();
    }

    #[endpoint(proxyDexAddPairToIntermediate)]
    fn proxy_dex_add_pair_to_intermediate(
        &self,
        proxy_dex_address: ManagedAddress,
        pair_address: ManagedAddress,
    ) {
        self.require_not_paused();

        let caller = self.blockchain().get_caller();
        self.require_deployed_contract(&caller, &proxy_dex_address);

        let _: () = self
            .proxy_dex_proxy()
            .contract(proxy_dex_address)
            .add_pair_to_intermediate(pair_address)
            .execute_on_dest_context();
    }

    #[endpoint(proxyDexRemoveIntermediatedPair)]
    fn proxy_dex_remove_intermediated_pair(
        &self,
        proxy_dex_address: ManagedAddress,
        pair_address: ManagedAddress,
    ) {
        self.require_not_paused();

        let caller = self.blockchain().get_caller();
        self.require_deployed_contract(&caller, &proxy_dex_address);

        let _: () = self
            .proxy_dex_proxy()
            .contract(proxy_dex_address)
            .remove_intermediated_pair(pair_address)
            .execute_on_dest_context();
    }

    #[endpoint(proxyDexAddFarmToIntermediate)]
    fn proxy_dex_add_farm_to_intermediate(
        &self,
        proxy_dex_address: ManagedAddress,
        farm_address: ManagedAddress,
    ) {
        self.require_not_paused();

        let caller = self.blockchain().get_caller();
        self.require_deployed_contract(&caller, &proxy_dex_address);

        let _: () = self
            .proxy_dex_proxy()
            .contract(proxy_dex_address)
            .add_farm_to_intermediate(farm_address)
            .execute_on_dest_context();
    }

    #[endpoint(proxyDexRemoveIntermediatedFarm)]
    fn proxy_dex_remove_intermediated_farm(
        &self,
        proxy_dex_address: ManagedAddress,
        farm_address: ManagedAddress,
    ) {
        self.require_not_paused();

        let caller = self.blockchain().get_caller();
        self.require_deployed_contract(&caller, &proxy_dex_address);

        let _: () = self
            .proxy_dex_proxy()
            .contract(proxy_dex_address)
            .remove_intermediated_farm(farm_address)
            .execute_on_dest_context();
    }

    #[proxy]
    fn proxy_dex_proxy(&self) -> proxy_dex::Proxy<Self::Api>;

    #[storage_mapper("proxyDexSource")]
    fn proxy_dex_source(&self) -> SingleValueMapper<ManagedAddress>;
}
