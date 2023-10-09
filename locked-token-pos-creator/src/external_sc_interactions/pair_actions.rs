multiversx_sc::imports!();

pub struct PairAddLiqResult<M: ManagedTypeApi> {
    pub lp_tokens: EsdtTokenPayment<M>,
    pub first_tokens_remaining: EsdtTokenPayment<M>,
    pub second_tokens_remaining: EsdtTokenPayment<M>,
}

#[multiversx_sc::module]
pub trait PairActionsModule {
    fn call_pair_swap(
        &self,
        input_tokens: EsdtTokenPayment,
        requested_token_id: TokenIdentifier,
        min_amount_out: BigUint,
    ) -> EsdtTokenPayment {
        let pair_address = self.mex_wegld_pair_address().get();
        self.pair_proxy(pair_address)
            .swap_tokens_fixed_input(requested_token_id, min_amount_out)
            .with_esdt_transfer(input_tokens)
            .execute_on_dest_context()
    }

    #[storage_mapper("mexWegldPairAddress")]
    fn mex_wegld_pair_address(&self) -> SingleValueMapper<ManagedAddress>;

    #[proxy]
    fn pair_proxy(&self, sc_address: ManagedAddress) -> pair::Proxy<Self::Api>;
}
