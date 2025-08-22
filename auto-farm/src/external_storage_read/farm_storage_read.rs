multiversx_sc::imports!();
multiversx_sc::derive_imports!();

use read_external_storage::State;

#[type_abi]
#[derive(TopEncode, TopDecode, Debug)]
pub struct FarmConfig<M: ManagedTypeApi> {
    pub state: State,
    pub farm_token_id: TokenIdentifier<M>,
    pub farming_token_id: TokenIdentifier<M>,
}

#[multiversx_sc::module]
pub trait FarmStorageReadModule:
    utils::UtilsModule + read_external_storage::ReadExternalStorageModule
{
    #[label("farm-whitelist-endpoints")]
    #[view(getFarmConfig)]
    fn get_farm_config(&self, farm_address: ManagedAddress) -> FarmConfig<Self::Api> {
        let state = self.get_farm_state(farm_address.clone());
        let farm_token_id = self.get_farm_token_id_mapper(farm_address.clone()).get();
        let farming_token_id = self.get_farming_token_id_mapper(farm_address).get();

        self.require_valid_token_id(&farm_token_id);
        self.require_valid_token_id(&farming_token_id);

        FarmConfig {
            state,
            farm_token_id,
            farming_token_id,
        }
    }

    #[inline]
    fn get_farm_state(&self, farm_address: ManagedAddress) -> State {
        self.get_farm_state_mapper(farm_address).get()
    }
}
