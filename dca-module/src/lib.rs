#![no_std]

multiversx_sc::imports!();

pub mod router_actions;
pub mod user_funds;

#[multiversx_sc::contract]
pub trait DcaModule:
    user_funds::UserFundsModule
    + router_actions::RouterActionsModule
    + utils::UtilsModule
    + multiversx_sc_modules::pause::PauseModule
{
    #[init]
    fn init(&self, router_address: ManagedAddress) {
        self.require_sc_address(&router_address);

        self.router_address().set(router_address);

        self.set_paused(true);
    }

    #[upgrade]
    fn upgrade(&self) {}
}
