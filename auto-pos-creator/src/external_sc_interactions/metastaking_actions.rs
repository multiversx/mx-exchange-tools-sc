elrond_wasm::imports!();

#[elrond_wasm::module]
pub trait MetastakingActionsModule {
    fn call_metastaking_stake(
        &self,
        sc_address: ManagedAddress,
        user: ManagedAddress,
        lp_farm_tokens: EsdtTokenPayment,
    ) -> EsdtTokenPayment {
        self.metastaking_proxy(sc_address)
            .stake_farm_tokens(user)
            .with_esdt_transfer(lp_farm_tokens)
            .execute_on_dest_context()
    }

    #[proxy]
    fn metastaking_proxy(&self, sc_address: ManagedAddress)
        -> farm_staking_proxy::Proxy<Self::Api>;
}
