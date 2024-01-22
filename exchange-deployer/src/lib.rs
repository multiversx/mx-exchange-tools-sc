#![no_std]

multiversx_sc::imports!();

pub mod action_type;
pub mod fee;

#[multiversx_sc::contract]
pub trait ExchangeDeployer: fee::FeeModule + multiversx_sc_modules::pause::PauseModule {
    #[init]
    fn init(&self, default_action_fee: BigUint) {
        self.set_paused(true);

        self.set_default_action_fee(default_action_fee);
    }

    #[upgrade]
    fn upgrade(&self) {}
}
