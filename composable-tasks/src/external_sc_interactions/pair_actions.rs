multiversx_sc::imports!();

use crate::errors::{
    ERROR_BACK_TRANSFERS_WRONG_PAYMENTS_NO, ERROR_WRONG_RETURNED_TOKEN_IDENTIFIER,
};

use super::router_actions;
use pair::pair_actions::swap::ProxyTrait as _;

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
        to_token_id: TokenIdentifier,
        min_amount_out: BigUint,
    ) -> EsdtTokenPayment {
        if from_tokens == to_token_id {
            return EsdtTokenPayment::new(from_tokens, 0, from_amount);
        }

        let pair_address = self.get_pair(from_tokens.clone(), to_token_id.clone());
        let payment = EsdtTokenPayment::new(from_tokens, 0, from_amount);

        let ((), back_transfers) = self
            .pair_proxy(pair_address)
            .swap_tokens_fixed_input(to_token_id.clone(), min_amount_out)
            .with_esdt_transfer(payment)
            .execute_on_dest_context_with_back_transfers();

        require!(
            back_transfers.esdt_payments.len() == 1,
            ERROR_BACK_TRANSFERS_WRONG_PAYMENTS_NO
        );

        let payment_out = back_transfers.esdt_payments.get(0).clone();
        require!(
            payment_out.token_identifier == to_token_id,
            ERROR_WRONG_RETURNED_TOKEN_IDENTIFIER
        );

        payment_out
    }

    fn perform_swap_tokens_fixed_output(
        &self,
        from_token_id: TokenIdentifier,
        from_amount: BigUint,
        to_token_id: TokenIdentifier,
        amount_out: BigUint,
    ) -> ManagedVec<EsdtTokenPayment> {
        if from_token_id == to_token_id {
            return ManagedVec::from_single_item(EsdtTokenPayment::new(
                from_token_id,
                0,
                from_amount,
            ));
        }

        let pair_address = self.get_pair(from_token_id.clone(), to_token_id.clone());
        let payment = EsdtTokenPayment::new(from_token_id, 0, from_amount);

        let ((), back_transfers) = self
            .pair_proxy(pair_address)
            .swap_tokens_fixed_output(to_token_id.clone(), amount_out)
            .with_esdt_transfer(payment)
            .execute_on_dest_context_with_back_transfers();

        require!(
            back_transfers.esdt_payments.len() <= 2,
            ERROR_BACK_TRANSFERS_WRONG_PAYMENTS_NO
        );

        let payment_out = back_transfers.esdt_payments.get(0).clone();
        require!(
            payment_out.token_identifier == to_token_id,
            ERROR_WRONG_RETURNED_TOKEN_IDENTIFIER
        );

        back_transfers.esdt_payments
    }

    #[proxy]
    fn pair_proxy(&self, sc_address: ManagedAddress) -> pair::Proxy<Self::Api>;
}
