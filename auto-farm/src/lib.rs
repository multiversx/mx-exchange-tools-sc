#![no_std]

elrond_wasm::imports!();

#[elrond_wasm::contract]
pub trait AutoFarm {
    #[init]
    fn init(&self) {}
}
