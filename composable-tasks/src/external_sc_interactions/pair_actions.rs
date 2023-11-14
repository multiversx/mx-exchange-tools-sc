multiversx_sc::imports!();

pub enum PairAddressForTokens<M: ManagedTypeApi> {
    CorrectOrder(ManagedAddress<M>),
    ReverseOrder(ManagedAddress<M>),
}

impl<M: ManagedTypeApi> PairAddressForTokens<M> {
    pub fn unwrap_address(self) -> ManagedAddress<M> {
        match self {
            PairAddressForTokens::CorrectOrder(addr) => addr,
            PairAddressForTokens::ReverseOrder(addr) => addr,
        }
    }

    pub fn is_reverse(&self) -> bool {
        matches!(self, PairAddressForTokens::ReverseOrder(_))
    }
}

#[multiversx_sc::module]
pub trait PairActionsModule {
    fn perform_tokens_swap(
        &self,
        from_tokens: TokenIdentifier,
        from_amount: BigUint,
        to_tokens: TokenIdentifier,
        min_amount_out: BigUint,
    ) -> EsdtTokenPayment {
        if from_tokens == to_tokens {
            return EsdtTokenPayment::new(from_tokens, 0, from_amount);
        }

        let pair_address = self
            .get_pair_address_for_tokens(&from_tokens, &to_tokens)
            .unwrap_address();
        let payment = EsdtTokenPayment::new(from_tokens, 0, from_amount);

        self.call_pair_swap(pair_address, payment, to_tokens, min_amount_out)
    }

    fn call_pair_swap(
        &self,
        pair_address: ManagedAddress,
        input_tokens: EsdtTokenPayment,
        requested_token_id: TokenIdentifier,
        min_amount_out: BigUint,
    ) -> EsdtTokenPayment {
        self.pair_proxy(pair_address)
            .swap_tokens_fixed_input(requested_token_id, min_amount_out)
            .with_esdt_transfer(input_tokens)
            .execute_on_dest_context()
    }

    fn get_pair_address_for_tokens(
        &self,
        first_token_id: &TokenIdentifier,
        second_token_id: &TokenIdentifier,
    ) -> PairAddressForTokens<Self::Api> {
        let correct_order_mapper = self.pair_address_for_tokens(first_token_id, second_token_id);
        if !correct_order_mapper.is_empty() {
            return PairAddressForTokens::CorrectOrder(correct_order_mapper.get());
        }

        let reverse_order_mapper = self.pair_address_for_tokens(second_token_id, first_token_id);
        require!(!reverse_order_mapper.is_empty(), "No pair for given tokens");

        PairAddressForTokens::ReverseOrder(reverse_order_mapper.get())
    }

    #[proxy]
    fn pair_proxy(&self, sc_address: ManagedAddress) -> pair::Proxy<Self::Api>;

    #[storage_mapper("pairAddrForTokens")]
    fn pair_address_for_tokens(
        &self,
        first_token_id: &TokenIdentifier,
        second_token_id: &TokenIdentifier,
    ) -> SingleValueMapper<ManagedAddress>;
}
