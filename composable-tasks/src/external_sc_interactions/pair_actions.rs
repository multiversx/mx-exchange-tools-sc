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
    fn perform_swap_tokens_fixed_input(
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

        self.pair_proxy(pair_address)
            .swap_tokens_fixed_input(to_tokens, min_amount_out)
            .with_esdt_transfer(payment)
            .execute_on_dest_context()
    }

    fn perform_swap_tokens_fixed_output(
        &self,
        from_tokens: TokenIdentifier,
        from_amount: BigUint,
        to_tokens: TokenIdentifier,
        amount_out: BigUint,
    ) -> ManagedVec<EsdtTokenPayment> {
        if from_tokens == to_tokens {
            return ManagedVec::from_single_item(EsdtTokenPayment::new(
                from_tokens,
                0,
                from_amount,
            ));
        }

        let pair_address = self.get_pair(from_tokens.clone(), to_tokens.clone());
        let payment = EsdtTokenPayment::new(from_tokens, 0, from_amount);

        let ((), back_transfers) = self
            .pair_proxy(pair_address)
            .swap_tokens_fixed_output(to_tokens, amount_out)
            .with_esdt_transfer(payment)
            .execute_on_dest_context_with_back_transfers();

        require!(
            back_transfers.esdt_payments.len() == 2,
            "Swap tokens fixed output: Back transfers expected 2 payments"
        );

        back_transfers.esdt_payments
    }

    #[proxy]
    fn pair_proxy(&self, sc_address: ManagedAddress) -> pair::Proxy<Self::Api>;
}
