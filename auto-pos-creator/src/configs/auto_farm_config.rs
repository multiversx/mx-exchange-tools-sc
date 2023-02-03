multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait AutoFarmConfigModule {
    #[storage_mapper("autoFarmScAddress")]
    fn auto_farm_sc_address(&self) -> SingleValueMapper<ManagedAddress>;
}
