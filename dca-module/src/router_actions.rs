use router::multi_pair_swap::ProxyTrait as _;

use crate::user_data::action::action_types::{ActionId, GasLimit, RouterSwapOperationType};

multiversx_sc::imports!();

pub const GAS_FOR_FINISH_EXECUTION: GasLimit = 10_000;

#[multiversx_sc::module]
pub trait RouterActionsModule: crate::user_data::action::storage::ActionStorageModule {
    fn call_router_swap(
        &self,
        action_id: ActionId,
        user_address: ManagedAddress,
        input_tokens: EsdtTokenPayment,
        swap_operations: MultiValueEncoded<RouterSwapOperationType<Self::Api>>,
    ) {
        let router_address = self.router_address().get();
        let gas_left = self.blockchain().get_gas_left();
        let promise_gas = gas_left - GAS_FOR_FINISH_EXECUTION;

        self.router_proxy(router_address)
            .multi_pair_swap(swap_operations)
            .with_esdt_transfer(input_tokens.clone())
            .with_gas_limit(promise_gas)
            .with_callback(
                self.callbacks()
                    .promise_callback(action_id, user_address, input_tokens),
            )
            .register_promise();
    }

    // TODO: Maybe some events in the callback
    #[promises_callback]
    fn promise_callback(
        &self,
        action_id: ActionId,
        user: ManagedAddress,
        original_tokens: EsdtTokenPayment,
        #[call_result] result: ManagedAsyncCallResult<IgnoreValue>,
    ) {
        let current_timestamp = self.blockchain().get_block_timestamp();
        let action_mapper = self.action_info(action_id);

        match result {
            ManagedAsyncCallResult::Ok(_) => {
                let actions_left = action_mapper.update(|action_info| {
                    action_info.total_actions_left -= 1;
                    action_info.last_action_timestamp = current_timestamp;
                    action_info.action_in_progress = false;

                    action_info.total_actions_left
                });

                if actions_left == 0 {
                    action_mapper.clear();
                }

                self.nr_retries_per_action(action_id).clear();

                let transfers = self.call_value().all_esdt_transfers().clone_value();
                if !transfers.is_empty() {
                    self.send().direct_multi(&user, &transfers);
                }
            }
            ManagedAsyncCallResult::Err(_) => {
                let nr_retries = self.nr_retries_per_action(action_id).get();
                let allowed_retries = self.nr_retries().get();
                if nr_retries <= allowed_retries {
                    action_mapper.update(|action_info| {
                        action_info.last_action_timestamp = current_timestamp;
                        action_info.action_in_progress = false;
                    });
                } else {
                    action_mapper.clear();
                }

                self.send().direct_esdt(
                    &user,
                    &original_tokens.token_identifier,
                    original_tokens.token_nonce,
                    &original_tokens.amount,
                );
            }
        }
    }

    #[proxy]
    fn router_proxy(&self, sc_address: ManagedAddress) -> router::Proxy<Self::Api>;

    #[storage_mapper("routerAddress")]
    fn router_address(&self) -> SingleValueMapper<ManagedAddress>;
}
