use super::unique_payments::UniquePayments;

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[type_abi]
#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, PartialEq, Debug)]
pub struct RewardsWrapper<M: ManagedTypeApi> {
    locked_token_id: TokenIdentifier<M>,
    pub locked_tokens: UniquePayments<M>,
    pub other_tokens: UniquePayments<M>,
}

impl<M: ManagedTypeApi> RewardsWrapper<M> {
    pub fn new(locked_token_id: TokenIdentifier<M>) -> Self {
        Self {
            locked_token_id,
            locked_tokens: UniquePayments::default(),
            other_tokens: UniquePayments::default(),
        }
    }

    pub fn add_tokens(&mut self, payment: EsdtTokenPayment<M>) {
        if payment.token_identifier == self.locked_token_id {
            self.locked_tokens.add_payment(payment);
        } else {
            self.other_tokens.add_payment(payment);
        }
    }

    #[inline]
    pub fn get_locked_token_id(&self) -> &TokenIdentifier<M> {
        &self.locked_token_id
    }
}
