multiversx_sc::imports!();

use crate::external_sc_interactions;

pub const SWAP_ARGS_LEN: usize = 3;
pub const ROUTER_SWAP_ARGS_LEN: usize = 4;
pub const SEND_TOKENS_ARGS_LEN: usize = 1;
pub const SWAP_TOKENS_FIXED_INPUT_FUNC_NAME: &[u8] = b"swapTokensFixedInput";
pub const SWAP_TOKENS_FIXED_OUTPUT_FUNC_NAME: &[u8] = b"swapTokensFixedOutput";

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
    #[endpoint(setRouterAddr)]
    fn set_router_address(&self, new_addr: ManagedAddress) {
        self.router_addr().set(new_addr);
    }
}
