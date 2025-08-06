multiversx_sc::imports!();

use pair::config::MAX_PERCENTAGE;

use crate::{errors::ERROR_WRONG_PERCENTAGE_AMOUNT, external_sc_interactions};

pub const SWAP_ARGS_LEN: usize = 3;
pub const ROUTER_SWAP_ARGS_LEN: usize = 4;
pub const SMART_SWAP_ARGS_LEN: usize = 5;
pub const SEND_TOKENS_ARGS_LEN: usize = 1;
pub const SWAP_TOKENS_FIXED_INPUT_FUNC_NAME: &[u8] = b"swapTokensFixedInput";
pub const SWAP_TOKENS_FIXED_OUTPUT_FUNC_NAME: &[u8] = b"swapTokensFixedOutput";
pub const SMART_SWAP_MIN_ARGS_LEN: usize = 1;
pub const ROUTER_TOKEN_OUT_FROM_END_OFFSET: usize = 2;
pub const SMART_SWAP_MAX_OPERATIONS: u64 = 10;

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

    #[only_owner]
    #[endpoint(setSmartSwapFeePercentage)]
    fn set_smart_swap_fee_percentage(&self, fee: u64) {
        require!(
            fee > 0 && fee < MAX_PERCENTAGE,
            ERROR_WRONG_PERCENTAGE_AMOUNT
        );
        self.smart_swap_fee_percentage().set(fee);
    }

    #[view(getSmartSwapFeePercentage)]
    #[storage_mapper("smartSwapFeePercentage")]
    fn smart_swap_fee_percentage(&self) -> SingleValueMapper<u64>;

    #[view(getSmartSwapFees)]
    #[storage_mapper("smartSwapFees")]
    fn smart_swap_fees(&self, token_id: &TokenIdentifier) -> SingleValueMapper<BigUint>;
}
