use super::unique_payments::UniquePayments;

elrond_wasm::imports!();
elrond_wasm::derive_imports!();

#[derive(TypeAbi, TopEncode, TopDecode, PartialEq, Debug)]
pub struct RewardsWrapper<M: ManagedTypeApi> {
    pub opt_locked_tokens: Option<EsdtTokenPayment<M>>,
    pub other_tokens: UniquePayments<M>,
}

impl<M: ManagedTypeApi> Default for RewardsWrapper<M> {
    #[inline]
    fn default() -> Self {
        Self {
            opt_locked_tokens: None,
            other_tokens: UniquePayments::default(),
        }
    }
}
