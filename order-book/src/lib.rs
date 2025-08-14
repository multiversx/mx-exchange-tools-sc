#![no_std]

multiversx_sc::imports!();

pub mod actors;
pub mod events;
pub mod pause;
pub mod storage;

#[multiversx_sc::contract]
pub trait OrderBook:
    storage::order::OrderModule
    + storage::common_storage::CommonStorageModule
    + pause::PauseModule
    + multiversx_sc_modules::only_admin::OnlyAdminModule
{
    #[init]
    fn init(&self) {}

    #[upgrade]
    fn upgrade(&self) {}
}
