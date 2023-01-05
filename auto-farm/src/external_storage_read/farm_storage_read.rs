elrond_wasm::imports!();
elrond_wasm::derive_imports!();

#[derive(
    TypeAbi, TopEncode, TopDecode, NestedEncode, NestedDecode, PartialEq, Copy, Clone, Debug,
)]
pub enum State {
    Inactive,
    Active,
    PartialActive,
}

#[derive(TypeAbi, TopEncode, TopDecode)]
pub struct FarmConfig<M: ManagedTypeApi> {
    pub state: State,
    pub farm_token_id: TokenIdentifier<M>,
    pub farming_token_id: TokenIdentifier<M>,
}

#[elrond_wasm::module]
pub trait FarmStorageReadModule: utils::UtilsModule {
    #[view(getFarmConfig)]
    fn get_farm_config(&self, farm_address: &ManagedAddress) -> FarmConfig<Self::Api> {
        let state = self.farm_state().get_from_address(farm_address);
        let farm_token_id = self.farm_token_id().get_from_address(farm_address);
        let farming_token_id = self.farming_token_id().get_from_address(farm_address);

        self.require_valid_token_id(&farm_token_id);
        self.require_valid_token_id(&farming_token_id);

        FarmConfig {
            state,
            farm_token_id,
            farming_token_id,
        }
    }

    #[inline]
    fn get_farm_state(&self, farm_address: &ManagedAddress) -> State {
        self.farm_state().get_from_address(farm_address)
    }

    #[storage_mapper("state")]
    fn farm_state(&self) -> SingleValueMapper<State>;

    #[storage_mapper("farm_token_id")]
    fn farm_token_id(&self) -> SingleValueMapper<TokenIdentifier>;

    #[storage_mapper("farming_token_id")]
    fn farming_token_id(&self) -> SingleValueMapper<TokenIdentifier>;
}
