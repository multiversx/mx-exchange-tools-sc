#![no_std]

multiversx_sc::imports!();

pub mod compose_tasks;
pub mod config;
pub mod errors;
pub mod events;
pub mod external_sc_interactions;
pub mod task_types;

#[multiversx_sc::contract]
pub trait ComposableTasksContract:
    compose_tasks::TaskCall
    + config::ConfigModule
    + external_sc_interactions::pair_actions::PairActionsModule
    + external_sc_interactions::router_actions::RouterActionsModule
    + external_sc_interactions::wegld_swap::WegldWrapModule
    + events::EventsModule
    + task_types::smart_swap::SmartSwapModule
{
    #[init]
    fn init(&self) {}

    #[upgrade]
    fn upgrade(&self) {}
}
