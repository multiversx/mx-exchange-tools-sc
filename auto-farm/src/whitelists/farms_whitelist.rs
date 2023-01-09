use common_structs::PaymentsVec;

use crate::common::address_to_id_mapper::{AddressId, AddressToIdMapper, NULL_ID};

elrond_wasm::imports!();

#[elrond_wasm::module]
pub trait FarmsWhitelistModule:
    crate::external_storage_read::farm_storage_read::FarmStorageReadModule + utils::UtilsModule
{
    /// Can also be used for farm-staking contracts.
    #[only_owner]
    #[endpoint(addFarms)]
    fn add_farms(&self, farms: MultiValueEncoded<ManagedAddress>) {
        let farms_mapper = self.farm_ids();
        for farm_addr in farms {
            self.require_sc_address(&farm_addr);

            let new_id = farms_mapper.insert_new(&farm_addr);
            let farm_config = self.get_farm_config(&farm_addr);
            self.farm_for_farm_token(&farm_config.farm_token_id)
                .set(new_id);

            let farming_token_mapper = self.farm_for_farming_token(&farm_config.farming_token_id);
            require!(
                farming_token_mapper.is_empty(),
                "Farming token already associated with another farm"
            );
            farming_token_mapper.set(new_id);
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
            self.farm_for_farming_token(&farm_config.farming_token_id)
                .clear();
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

    #[view(getFarmForFarmingToken)]
    fn get_farm_for_farming_token_view(
        &self,
        farming_token_id: TokenIdentifier,
    ) -> OptionalValue<ManagedAddress> {
        let farm_id = self.farm_for_farming_token(&farming_token_id).get();
        self.farm_ids().get_address(farm_id).into()
    }

    fn get_farm_ids_for_farm_tokens(
        &self,
        user_farm_tokens: &PaymentsVec<Self::Api>,
    ) -> ManagedVec<AddressId> {
        let mut ids = ManagedVec::new();
        for farm_token in user_farm_tokens {
            let farm_id = self.farm_for_farm_token(&farm_token.token_identifier).get();
            ids.push(farm_id);
        }

        ids
    }

    #[storage_mapper("farmIds")]
    fn farm_ids(&self) -> AddressToIdMapper<Self::Api>;

    #[storage_mapper("farmForFarmToken")]
    fn farm_for_farm_token(&self, farm_token_id: &TokenIdentifier) -> SingleValueMapper<AddressId>;

    #[storage_mapper("farmForFarmingToken")]
    fn farm_for_farming_token(
        &self,
        farming_token_id: &TokenIdentifier,
    ) -> SingleValueMapper<AddressId>;
}
