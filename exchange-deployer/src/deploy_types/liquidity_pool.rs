multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait LiquidityPoolModule: super::common::CommonModule + utils::UtilsModule {
    #[only_owner]
    #[endpoint(setRouterAddress)]
    fn set_router_address(&self, router_address: ManagedAddress) {
        self.require_sc_address(&router_address);

        self.router_address().set(router_address);
    }

    #[only_owner]
    #[endpoint(setRouterOwnerAddress)]
    fn set_router_owner_address(&self, router_owner_address: ManagedAddress) {
        self.router_owner_address().set(router_owner_address);
    }

    #[only_owner]
    #[endpoint(setPairSourceAddress)]
    fn set_pair_source_address(&self, pair_source: ManagedAddress) {
        self.require_sc_address(&pair_source);

        self.pair_source().set(pair_source);
    }

    #[payable("*")]
    #[endpoint(deployLiquidityPool)]
    fn deploy_liquidity_pool(
        &self,
        first_token_id: TokenIdentifier,
        second_token_id: TokenIdentifier,
        total_fee_percent: u64,
        special_fee_percent: u64,
    ) {
        let router_address = self.router_address().get();
        let router_owner_address = self.router_owner_address().get();
        let caller = self.blockchain().get_caller();
        let mut admins = MultiValueEncoded::new();
        admins.push(caller.clone());

        let pair_source = self.pair_source().get();
        let code_metadata = self.get_default_code_metadata();
        let (deployed_sc_address, ()) = self
            .pair_proxy()
            .init(
                first_token_id,
                second_token_id,
                router_address,
                router_owner_address,
                total_fee_percent,
                special_fee_percent,
                caller.clone(),
                admins,
            )
            .deploy_from_source(&pair_source, code_metadata);

        let _ = self.deployed_contracts(&caller).insert(deployed_sc_address);
    }

    #[proxy]
    fn pair_proxy(&self) -> pair::Proxy<Self::Api>;

    #[storage_mapper("pairSource")]
    fn pair_source(&self) -> SingleValueMapper<ManagedAddress>;

    #[storage_mapper("routerAddress")]
    fn router_address(&self) -> SingleValueMapper<ManagedAddress>;

    #[storage_mapper("routerOwnerAddress")]
    fn router_owner_address(&self) -> SingleValueMapper<ManagedAddress>;
}
