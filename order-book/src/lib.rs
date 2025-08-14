#![no_std]

use crate::storage::common_storage::Percent;

multiversx_sc::imports!();

pub mod actors;
pub mod events;
pub mod pause;
pub mod storage;

#[multiversx_sc::contract]
pub trait OrderBook:
    actors::admin::AdminModule
    + storage::order::OrderModule
    + storage::common_storage::CommonStorageModule
    + pause::PauseModule
    + utils::UtilsModule
    + multiversx_sc_modules::only_admin::OnlyAdminModule
{
    #[init]
    fn init(
        &self,
        router_address: ManagedAddress,
        treasury_address: ManagedAddress,
        pruning_fee: Percent,
        p2p_protocol_fee: Percent,
        admins: MultiValueEncoded<ManagedAddress>,
    ) {
        self.set_router_address(router_address);
        self.set_treasury_address(treasury_address);
        self.set_pruning_fee(pruning_fee);
        self.set_p2p_protocol_fee(p2p_protocol_fee);

        for admin in admins {
            self.add_admin(admin);
        }

        self.set_paused(true);
    }

    #[upgrade]
    fn upgrade(&self) {}
}
