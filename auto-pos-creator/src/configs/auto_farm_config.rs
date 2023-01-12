elrond_wasm::imports!();

#[elrond_wasm::module]
pub trait AutoFarmConfigModule {
    #[storage_mapper("autoFarmScAddress")]
    fn auto_farm_sc_address(&self) -> SingleValueMapper<ManagedAddress>;
}
