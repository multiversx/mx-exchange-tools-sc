multiversx_sc::imports!();

use crate::external_sc_interactions;

#[multiversx_sc::module]
pub trait ConfigModule:
    external_sc_interactions::pair_actions::PairActionsModule
    + external_sc_interactions::router_actions::RouterActionsModule
    + external_sc_interactions::wegld_swap::WegldWrapModule
{
    #[only_owner]
    #[endpoint(setWrapEgldAddr)]
    fn set_wrap_egld_address(&self, new_addr: ManagedAddress) {
        self.wrap_egld_addr().set(new_addr);
    }

    #[only_owner]
    #[endpoint(setrouterAddr)]
    fn set_router_address(&self, new_addr: ManagedAddress) {
        self.router_addr().set(new_addr);
    }

    #[only_owner]
    #[endpoint(setPairAddrForTokens)]
    fn set_pair_address_for_tokens(
        &self,
        first_token_id: &TokenIdentifier,
        second_token_id: &TokenIdentifier,
        new_addr: ManagedAddress,
    ) {
        self.pair_address_for_tokens(first_token_id, second_token_id)
            .set(new_addr);
    }
}
