use common_structs::PaymentsVec;
use mergeable::Mergeable;
use multiversx_sc::api::HandleConstraints;
use multiversx_sc::api::{SendApi, SendApiImpl};

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[derive(TypeAbi, TopEncode, TopDecode, NestedEncode, NestedDecode, Clone, PartialEq, Debug)]
pub struct PaymentsWrapper<M: ManagedTypeApi> {
    payments: PaymentsVec<M>,
}

impl<M: ManagedTypeApi> Default for PaymentsWrapper<M> {
    #[inline]
    fn default() -> Self {
        Self {
            payments: Default::default(),
        }
    }
}

impl<M: ManagedTypeApi> PaymentsWrapper<M>
where
    M: ManagedTypeApi,
{
    #[inline]
    pub fn new() -> Self {
        Self {
            payments: ManagedVec::new(),
        }
    }

    pub fn push(&mut self, payment: EsdtTokenPayment<M>) {
        if payment.amount == 0 {
            return;
        }

        self.payments.push(payment);
    }

    pub fn into_payments(self) -> PaymentsVec<M> {
        self.payments
    }

    pub fn iter(&self) -> ManagedVecRefIterator<M, EsdtTokenPayment<M>> {
        self.payments.iter()
    }
}

impl<M> PaymentsWrapper<M>
where
    M: ManagedTypeApi + SendApi,
{
    pub fn send_to(&self, address: &ManagedAddress<M>) {
        if self.payments.is_empty() {
            return;
        }

        let _ = M::send_api_impl().multi_transfer_esdt_nft_execute(
            address.get_handle().get_raw_handle(),
            self.payments.get_handle().get_raw_handle(),
            0,
            ManagedBuffer::<M>::new().get_handle().get_raw_handle(),
            ManagedArgBuffer::<M>::new().get_handle().get_raw_handle(),
        );
    }
}

impl<M: ManagedTypeApi> Mergeable<M> for PaymentsWrapper<M> {
    /// Can always be merged
    #[inline]
    fn can_merge_with(&self, _other: &Self) -> bool {
        true
    }

    fn merge_with(&mut self, other: Self) {
        self.error_if_not_mergeable(&other);

        self.payments.append_vec(other.payments);
    }
}
