use pair::pair_actions::swap::ProxyTrait as _;

multiversx_sc::imports!();

pub type GasLimit = u64;

pub const SWAP_GAS_LIMIT: GasLimit = 25_000_000;

pub struct PairSwapArgs<M: ManagedTypeApi> {
    pub pair_address: ManagedAddress<M>,
    pub input_tokens: EsdtTokenPayment<M>,
    pub requested_token_id: TokenIdentifier<M>,
    pub user_address: ManagedAddress<M>,
    pub swap_min_amount: BigUint<M>,
}

// TODO: Change to use router endpoint instead

#[multiversx_sc::module]
pub trait PairActionsModule {
    fn call_pair_swap_promise(&self, args: PairSwapArgs<Self::Api>) {
        self.pair_proxy(args.pair_address)
            .swap_tokens_fixed_input(args.requested_token_id, args.swap_min_amount)
            .with_esdt_transfer(args.input_tokens.clone())
            .callback(
                self.callbacks()
                    .promise_callback(args.user_address, args.input_tokens),
            )
            .with_gas_limit(SWAP_GAS_LIMIT)
            .register_promise();
    }

    #[promises_callback]
    fn promise_callback(
        &self,
        user: ManagedAddress,
        original_tokens: EsdtTokenPayment,
        #[call_result] result: ManagedAsyncCallResult<IgnoreValue>,
    ) {
        match result {
            ManagedAsyncCallResult::Ok(_) => {
                let esdt_transfer = self.call_value().single_esdt();
                self.send().direct_esdt(
                    &user,
                    &esdt_transfer.token_identifier,
                    esdt_transfer.token_nonce,
                    &esdt_transfer.amount,
                );
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
    fn pair_proxy(&self, sc_address: ManagedAddress) -> pair::Proxy<Self::Api>;
}
