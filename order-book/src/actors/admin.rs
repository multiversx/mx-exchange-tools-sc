use crate::storage::common_storage::{Percent, MAX_PERCENT};

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait AdminModule:
    crate::storage::common_storage::CommonStorageModule
    + crate::pause::PauseModule
    + utils::UtilsModule
    + multiversx_sc_modules::only_admin::OnlyAdminModule
{
    #[only_admin]
    #[endpoint(pause)]
    fn pause_endpoint(&self) {
        self.set_paused(true);
        self.pause_event();
    }

    #[only_admin]
    #[endpoint(unpause)]
    fn unpause_endpoint(&self) {
        self.set_paused(false);
        self.unpause_event();
    }

    #[only_admin]
    #[endpoint(setRouterAddress)]
    fn set_router_address(&self, router_address: ManagedAddress) {
        self.require_sc_address(&router_address);

        self.router_address().set(router_address);
    }

    #[only_admin]
    #[endpoint(setTreasuryAddress)]
    fn set_treasury_address(&self, treasury_address: ManagedAddress) {
        self.treasury_address().set(treasury_address);
    }

    #[only_admin]
    #[endpoint(setPruningFee)]
    fn set_pruning_fee(&self, pruning_fee: Percent) {
        self.require_valid_percent(pruning_fee);

        self.pruning_fee().set(pruning_fee);
    }

    #[only_admin]
    #[endpoint(setP2pProtocolFee)]
    fn set_p2p_protocol_fee(&self, protocol_fee: Percent) {
        self.require_valid_percent(protocol_fee);

        self.p2p_protocol_fee().set(protocol_fee);
    }

    fn require_valid_percent(&self, percent: Percent) {
        require!(percent <= MAX_PERCENT, "Invalid percent");
    }
}
