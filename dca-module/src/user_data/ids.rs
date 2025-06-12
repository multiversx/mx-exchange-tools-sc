multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait IdsModule {
    #[storage_mapper("userId")]
    fn user_ids(&self) -> AddressToIdMapper<Self::Api>;
}
