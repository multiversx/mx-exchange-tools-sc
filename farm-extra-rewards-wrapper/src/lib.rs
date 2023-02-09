#![no_std]

multiversx_sc::imports!();

#[multiversx_sc::contract]
pub trait FarmExtraRewardsWrapper {
    #[init]
    fn init(&self) {}
}
