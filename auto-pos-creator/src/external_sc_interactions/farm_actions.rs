use farm::EnterFarmResultType;

elrond_wasm::imports!();

#[elrond_wasm::module]
pub trait FarmActionsModule {
    fn call_enter_farm(
        &self,
        farm_address: ManagedAddress,
        user: ManagedAddress,
        farming_tokens: EsdtTokenPayment,
    ) -> EsdtTokenPayment {
        let raw_results: EnterFarmResultType<Self::Api> = self
            .farm_proxy(farm_address)
            .enter_farm_endpoint(user)
            .with_esdt_transfer(farming_tokens)
            .execute_on_dest_context();

        // no rewards on simple enter
        let (new_farm_token, _) = raw_results.into_tuple();
        new_farm_token
    }

    #[proxy]
    fn farm_proxy(&self, sc_address: ManagedAddress) -> farm_with_locked_rewards::Proxy<Self::Api>;
}
