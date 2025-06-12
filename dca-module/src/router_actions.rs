use router::multi_pair_swap::ProxyTrait as _;

multiversx_sc::imports!();

/// Pairs of (pair address, endpoint name, requested token, min amount out)
pub type SwapOperationType<M> =
    MultiValue4<ManagedAddress<M>, ManagedBuffer<M>, TokenIdentifier<M>, BigUint<M>>;
pub type GasLimit = u64;

pub static SWAP_TOKENS_FIXED_INPUT_FUNC_NAME: &[u8] = b"swapTokensFixedInput";
pub const GAS_FOR_FINISH_EXECUTION: GasLimit = 10_000;

#[multiversx_sc::module]
pub trait RouterActionsModule {
    // TODO: Force fixed input swap
    // TODO: +1 tries per action. Lock until callback is called to prevent replaying the same action too many times
    fn call_router_swap(
        &self,
        user_address: ManagedAddress,
        input_tokens: EsdtTokenPayment,
        swap_operations: MultiValueEncoded<SwapOperationType<Self::Api>>,
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
                    .promise_callback(user_address, input_tokens),
            )
            .register_promise();
    }

    // TODO: Handle case of success (i.e. -1 action for user) and error (i.e. 3 retries, then mark as failed)
    #[promises_callback]
    fn promise_callback(
        &self,
        user: ManagedAddress,
        original_tokens: EsdtTokenPayment,
        #[call_result] result: ManagedAsyncCallResult<IgnoreValue>,
    ) {
        match result {
            ManagedAsyncCallResult::Ok(_) => {
                let transfers = self.call_value().all_esdt_transfers().clone_value();
                if !transfers.is_empty() {
                    self.send().direct_multi(&user, &transfers);
                }
            }
            ManagedAsyncCallResult::Err(_) => {
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
