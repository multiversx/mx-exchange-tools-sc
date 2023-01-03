use crate::address_to_id_mapper::{AddressId, AddressToIdMapper, NULL_ID};

elrond_wasm::imports!();

#[elrond_wasm::module]
pub trait FarmsWhitelistModule:
    crate::farm_external_storage_read::FarmExternalStorageReadModule + utils::UtilsModule
{
    #[only_owner]
    #[endpoint(addFarms)]
    fn add_farms(&self, farms: MultiValueEncoded<ManagedAddress>) {
        let farms_mapper = self.farm_ids();
        for farm_addr in farms {
            self.require_sc_address(&farm_addr);

            let existing_id = farms_mapper.get_id(&farm_addr);
            if existing_id != NULL_ID {
                continue;
            }

            let new_id = farms_mapper.get_id_or_insert(&farm_addr);
            let farm_config = self.get_farm_config(&farm_addr);
            self.farm_for_farm_token(&farm_config.farm_token_id)
                .set(new_id);
            let _ = self
                .farms_for_farming_token(&farm_config.farming_token_id)
                .insert(new_id);
        }
    }

    #[only_owner]
    #[endpoint(removeFarms)]
    fn remove_farms(&self, farms: MultiValueEncoded<ManagedAddress>) {
        let farms_mapper = self.farm_ids();
        for farm_addr in farms {
            let prev_id = farms_mapper.remove_by_address(&farm_addr);
            if prev_id == NULL_ID {
                continue;
            }

            let farm_config = self.get_farm_config(&farm_addr);
            self.farm_for_farm_token(&farm_config.farm_token_id).clear();
            let _ = self
                .farms_for_farming_token(&farm_config.farming_token_id)
                .swap_remove(&prev_id);
        }
    }

    #[view(getFarmForFarmToken)]
    fn get_farm_for_farm_token_view(
        &self,
        farm_token_id: TokenIdentifier,
    ) -> OptionalValue<ManagedAddress> {
        let farm_id = self.farm_for_farm_token(&farm_token_id).get();
        self.farm_ids().get_address(farm_id).into()
    }

    #[view(getFarmsForFarmingToken)]
    fn get_farms_for_farming_token_view(
        &self,
        farming_token_id: TokenIdentifier,
    ) -> MultiValueEncoded<ManagedAddress> {
        let ids_mapper = self.farm_ids();
        let mut results = MultiValueEncoded::new();
        for farm_id in self.farms_for_farming_token(&farming_token_id).iter() {
            let opt_farm_addr = ids_mapper.get_address(farm_id);
            if let Some(farm_addr) = opt_farm_addr {
                results.push(farm_addr);
            }
        }

        results
    }

    #[storage_mapper("farmIds")]
    fn farm_ids(&self) -> AddressToIdMapper<Self::Api>;

    #[storage_mapper("farmForFarmToken")]
    fn farm_for_farm_token(&self, farm_token_id: &TokenIdentifier) -> SingleValueMapper<AddressId>;

    #[storage_mapper("farmsForFarmingToken")]
    fn farms_for_farming_token(
        &self,
        farming_token_id: &TokenIdentifier,
    ) -> UnorderedSetMapper<AddressId>;
}
