#![no_std]

multiversx_sc::imports!();

pub mod external_sc_interactions;
pub mod compose_tasks;
pub mod config;

#[multiversx_sc::contract]
pub trait ComposableTasksContract:
    compose_tasks::TaskCall
    + config::ConfigModule
    + external_sc_interactions::pair_actions::PairActionsModule
    + external_sc_interactions::router_actions::RouterActionsModule
    + external_sc_interactions::wegld_swap::WegldWrapModule
{
    #[init]
    fn init(&self) {}
}
