multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait MetastakingWhitelistModule:
    read_external_storage::ReadExternalStorageModule
    + crate::external_storage_read::metastaking_storage_read::MetastakingStorageReadModule
    + utils::UtilsModule
{
    #[label("metastaking-whitelist-endpoints")]
    #[only_owner]
    #[endpoint(addMetastakingScs)]
    fn add_metastaking_scs(&self, scs: MultiValueEncoded<ManagedAddress>) {
        let ids_mapper = self.metastaking_ids();
        for sc_addr in scs {
            self.require_sc_address(&sc_addr);

            let new_id = ids_mapper.insert_new(&sc_addr);
            let ms_config = self.get_metastaking_config(sc_addr);
            self.metastaking_for_dual_yield_token(&ms_config.dual_yield_token_id)
                .set(new_id);
            self.metastaking_for_lp_farm_token(&ms_config.lp_farm_token_id)
                .set(new_id);
        }
    }

    #[label("metastaking-whitelist-endpoints")]
    #[only_owner]
    #[endpoint(removeMetastakingScs)]
    fn remove_metastaking_scs(&self, scs: MultiValueEncoded<ManagedAddress>) {
        let ids_mapper = self.metastaking_ids();
        for sc_addr in scs {
            let prev_id = ids_mapper.remove_by_address(&sc_addr);
            if prev_id == NULL_ID {
                continue;
            }

            let ms_config = self.get_metastaking_config(sc_addr);
            self.metastaking_for_dual_yield_token(&ms_config.dual_yield_token_id)
                .clear();
            self.metastaking_for_lp_farm_token(&ms_config.lp_farm_token_id)
                .clear();
        }
    }

    #[label("metastaking-whitelist-endpoints")]
    #[view(getMetastakingForDualYieldToken)]
    fn get_metastaking_for_dual_yield_token_view(
        &self,
        dual_yield_token_id: TokenIdentifier,
    ) -> OptionalValue<ManagedAddress> {
        let ms_id = self
            .metastaking_for_dual_yield_token(&dual_yield_token_id)
            .get();
        self.metastaking_ids().get_address(ms_id).into()
    }

    #[label("metastaking-whitelist-endpoints")]
    #[view(getMetastakingForLpFarmToken)]
    fn get_metastaking_for_lp_farm_token(
        &self,
        lp_farm_token_id: TokenIdentifier,
    ) -> OptionalValue<ManagedAddress> {
        let ms_id = self.metastaking_for_lp_farm_token(&lp_farm_token_id).get();
        self.metastaking_ids().get_address(ms_id).into()
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
    ) -> SingleValueMapper<AddressId>;
}
