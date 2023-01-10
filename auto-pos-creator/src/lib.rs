#![no_std]

elrond_wasm::imports!();

#[elrond_wasm::contract]
pub trait AutoPosCreator {
    #[init]
    fn init(&self) {}
}
