use crate::common::address_to_id_mapper::{AddressId, AddressToIdMapper, NULL_ID};

elrond_wasm::imports!();

#[elrond_wasm::module]
pub trait MetastakingWhitelistModule:
    crate::external_storage_read::metastaking_storage_read::MetastakingStorageReadModule
    + utils::UtilsModule
{
    #[only_owner]
    #[endpoint(addMetastakingScs)]
    fn add_metastaking_scs(&self, scs: MultiValueEncoded<ManagedAddress>) {
        let ids_mapper = self.metastaking_ids();
        for sc_addr in scs {
            self.require_sc_address(&sc_addr);

            let new_id = ids_mapper.insert_new(&sc_addr);
            let ms_config = self.get_metastaking_config(&sc_addr);
            self.metastaking_for_dual_yield_token(&ms_config.dual_yield_token_id)
                .set(new_id);
            let _ = self
                .metastaking_for_lp_farm_token(&ms_config.lp_farm_token_id)
                .insert(new_id);
        }
    }

    #[only_owner]
    #[endpoint(removeMetastakingScs)]
    fn remove_metastaking_scs(&self, scs: MultiValueEncoded<ManagedAddress>) {
        let ids_mapper = self.metastaking_ids();
        for sc_addr in scs {
            let prev_id = ids_mapper.remove_by_address(&sc_addr);
            if prev_id == NULL_ID {
                continue;
            }

            let ms_config = self.get_metastaking_config(&sc_addr);
            self.metastaking_for_dual_yield_token(&ms_config.dual_yield_token_id)
                .clear();
            let _ = self
                .metastaking_for_lp_farm_token(&ms_config.lp_farm_token_id)
                .swap_remove(&prev_id);
        }
    }

    #[view(getMetastakingScForDualYieldToken)]
    fn get_metastaking_for_dual_yield_token_view(
        &self,
        dual_yield_token_id: TokenIdentifier,
    ) -> OptionalValue<ManagedAddress> {
        let ms_id = self
            .metastaking_for_dual_yield_token(&dual_yield_token_id)
            .get();
        self.metastaking_ids().get_address(ms_id).into()
    }

    #[view(getMetastakingScsForLpFarmToken)]
    fn get_metastaking_scs_for_lp_farm_token(
        &self,
        lp_farm_token_id: TokenIdentifier,
    ) -> MultiValueEncoded<ManagedAddress> {
        let ids_mapper = self.metastaking_ids();
        let mut results = MultiValueEncoded::new();
        for farm_id in self.metastaking_for_lp_farm_token(&lp_farm_token_id).iter() {
            let opt_ms_addr = ids_mapper.get_address(farm_id);
            if let Some(ms_addr) = opt_ms_addr {
                results.push(ms_addr);
            }
        }

        results
    }

    #[storage_mapper("MSIds")]
    fn metastaking_ids(&self) -> AddressToIdMapper<Self::Api>;

    #[storage_mapper("MSForDYToken")]
    fn metastaking_for_dual_yield_token(
        &self,
        dual_yield_token_id: &TokenIdentifier,
    ) -> SingleValueMapper<AddressId>;

    #[storage_mapper("MSForLpFarmToken")]
    fn metastaking_for_lp_farm_token(
        &self,
        lp_farm_token_id: &TokenIdentifier,
    ) -> UnorderedSetMapper<AddressId>;
}
