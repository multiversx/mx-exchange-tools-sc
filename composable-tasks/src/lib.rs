#![no_std]

multiversx_sc::imports!();

pub mod external_sc_interactions;
pub mod compose_tasks;
pub mod config;

/// An empty contract. To be used as a template when starting a new contract from scratch.
#[multiversx_sc::contract]
pub trait ComposableTasksContract:
    compose_tasks::TaskCall
    + config::ConfigModule
    + external_sc_interactions::pair_actions::PairActionsModule
    + external_sc_interactions::router_actions::RouterActionsModule
    + external_sc_interactions::wegld_swap::WegldSwapModule
{
    #[init]
    fn init(&self) {}
}
