#![no_std]

multiversx_sc::imports!();

#[multiversx_sc::contract]
pub trait ExchangeDeployer {
    #[init]
    fn init(&self) {}
}
