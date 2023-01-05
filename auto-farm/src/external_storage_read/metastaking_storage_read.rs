elrond_wasm::imports!();
elrond_wasm::derive_imports!();

#[derive(TypeAbi, TopEncode, TopDecode)]
pub struct MetastakingConfig<M: ManagedTypeApi> {
    pub dual_yield_token_id: TokenIdentifier<M>,
    pub lp_farm_token_id: TokenIdentifier<M>,
}

#[elrond_wasm::module]
pub trait MetastakingStorageReadModule: utils::UtilsModule {
    #[view(getMetastakingConfig)]
    fn get_metastaking_config(
        &self,
        metastaking_address: &ManagedAddress,
    ) -> MetastakingConfig<Self::Api> {
        let dual_yield_token_id = self
            .dual_yield_token_id()
            .get_from_address(metastaking_address);
        let lp_farm_token_id = self
            .lp_farm_token_id()
            .get_from_address(metastaking_address);

        self.require_valid_token_id(&dual_yield_token_id);
        self.require_valid_token_id(&lp_farm_token_id);

        MetastakingConfig {
            dual_yield_token_id,
            lp_farm_token_id,
        }
    }

    #[storage_mapper("dualYieldTokenId")]
    fn dual_yield_token_id(&self) -> SingleValueMapper<TokenIdentifier>;

    #[storage_mapper("lpFarmTokenId")]
    fn lp_farm_token_id(&self) -> SingleValueMapper<TokenIdentifier>;
}
