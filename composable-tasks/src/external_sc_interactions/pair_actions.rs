use super::router_actions;

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
pub trait PairActionsModule: router_actions::RouterActionsModule {
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

        let pair_address = self.get_pair(from_tokens.clone(), to_tokens.clone());
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

    #[proxy]
    fn pair_proxy(&self, sc_address: ManagedAddress) -> pair::Proxy<Self::Api>;
}
