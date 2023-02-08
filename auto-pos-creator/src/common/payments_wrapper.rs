use common_structs::PaymentsVec;

multiversx_sc::imports!();

pub struct PaymentsWrapper<M: SendApi> {
    payments: PaymentsVec<M>,
}

impl<M: SendApi> Default for PaymentsWrapper<M> {
    #[inline]
    fn default() -> Self {
        Self {
            payments: PaymentsVec::new(),
        }
    }
}

impl<M: SendApi> PaymentsWrapper<M> {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, payment: EsdtTokenPayment<M>) {
        if payment.amount == 0 {
            return;
        }

        self.payments.push(payment);
    }

    pub fn send_and_return(self, to: &ManagedAddress<M>) -> PaymentsVec<M> {
        if self.payments.is_empty() {
            return self.payments;
        }

        let _ = M::send_api_impl().multi_transfer_esdt_nft_execute(
            to,
            &self.payments,
            0,
            &ManagedBuffer::new(),
            &ManagedArgBuffer::new(),
        );

        self.payments
    }
}
