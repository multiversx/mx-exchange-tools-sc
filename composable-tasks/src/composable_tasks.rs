#![no_std]

multiversx_sc::imports!();

pub mod external_sc_interactions;
mod task_call;

/// An empty contract. To be used as a template when starting a new contract from scratch.
#[multiversx_sc::contract]
pub trait ComposableTasksContract:
    task_call::TaskCall
    + external_sc_interactions::pair_actions::PairActionsModule
    + external_sc_interactions::farm_actions::FarmActionsModule
    + external_sc_interactions::wrap_egld::WrapEgldModule
{
    #[init]
    fn init(&self) {}
}
