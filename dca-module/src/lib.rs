#![no_std]

multiversx_sc::imports!();

pub mod user_funds;

#[multiversx_sc::contract]
pub trait DcaModule:
    user_funds::UserFundsModule + multiversx_sc_modules::pause::PauseModule
{
    #[init]
    fn init(&self) {
        self.set_paused(true);
    }

    #[upgrade]
    fn upgrade(&self) {}
}
