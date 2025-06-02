#![no_std]

multiversx_sc::imports!();

pub mod user_funds;

#[multiversx_sc::contract]
pub trait DcaModule: user_funds::UserFundsModule {
    #[init]
    fn init(&self) {}

    #[upgrade]
    fn upgrade(&self) {}
}
