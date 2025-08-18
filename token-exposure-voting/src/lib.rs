#![no_std]

use week_timekeeping::Epoch;

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

pub mod config;
pub mod views;
pub mod vote;

#[multiversx_sc::contract]
pub trait TokenExposureVotingModule:
    crate::config::ConfigModule
    + crate::vote::VoteModule
    + crate::views::ViewsModule
    + week_timekeeping::WeekTimekeepingModule
    + energy_query::EnergyQueryModule
{
    #[init]
    fn init(
        &self,
        first_week_start_epoch: Epoch,
        energy_factory_address: ManagedAddress,
        voting_token_id: TokenIdentifier,
    ) {
        self.first_week_start_epoch().set(first_week_start_epoch);
        self.set_energy_factory_address(energy_factory_address);
        self.voting_token_id().set(&voting_token_id);
    }

    #[upgrade]
    fn upgrade(&self) {}
}
