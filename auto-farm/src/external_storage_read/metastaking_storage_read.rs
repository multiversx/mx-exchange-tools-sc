multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[derive(TypeAbi, TopEncode, TopDecode)]
pub struct MetastakingConfig<M: ManagedTypeApi> {
    pub dual_yield_token_id: TokenIdentifier<M>,
    pub lp_farm_token_id: TokenIdentifier<M>,
}

#[multiversx_sc::module]
pub trait MetastakingStorageReadModule:
    read_external_storage::ReadExternalStorageModule + utils::UtilsModule
{
    #[label("metastaking-whitelist-endpoints")]
    #[view(getMetastakingConfig)]
    fn get_metastaking_config(
        &self,
        metastaking_address: ManagedAddress,
    ) -> MetastakingConfig<Self::Api> {
        let dual_yield_token_id = self
            .get_dual_yield_token_id_mapper(metastaking_address.clone())
            .get();
        let lp_farm_token_id = self.get_lp_farm_token_id_mapper(metastaking_address).get();

        self.require_valid_token_id(&dual_yield_token_id);
        self.require_valid_token_id(&lp_farm_token_id);

        MetastakingConfig {
            dual_yield_token_id,
            lp_farm_token_id,
        }
    }
}
