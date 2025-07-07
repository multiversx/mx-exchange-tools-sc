#![no_std]

use crate::user_data::action::action_types::NrRetries;

multiversx_sc::imports!();

pub mod events;
pub mod router_actions;
pub mod user_data;

#[multiversx_sc::contract]
pub trait DcaModule:
    user_data::ids::IdsModule
    + user_data::funds::FundsModule
    + user_data::action::user_action::ActionModule
    + user_data::action::edit_action::EditActionModule
    + user_data::action::storage::ActionStorageModule
    + router_actions::RouterActionsModule
    + events::EventsModule
    + utils::UtilsModule
    + multiversx_sc_modules::pause::PauseModule
{
    #[init]
    fn init(&self, router_address: ManagedAddress, nr_action_retries: NrRetries) {
        self.require_sc_address(&router_address);

        self.router_address().set(router_address);
        self.set_nr_retries(nr_action_retries);

        self.set_paused(true);
    }

    #[upgrade]
    fn upgrade(&self) {}
}
