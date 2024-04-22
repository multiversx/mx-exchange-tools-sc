multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait WegldWrapModule {
    fn wrap_egld(&self, payment: EgldOrEsdtTokenPayment) -> EgldOrEsdtTokenPayment {
        require!(
            payment.token_identifier.is_egld(),
            "Payment token is not EGLD!"
        );

        let wrap_egld_addr = self.wrap_egld_addr().get();

        let ((), back_transfers) = self
            .wrap_egld_proxy(wrap_egld_addr)
            .wrap_egld()
            .with_egld_transfer(payment.amount)
            .execute_on_dest_context_with_back_transfers();

        let returned_wrapped_egld = back_transfers.esdt_payments;
        require!(
            returned_wrapped_egld.len() == 1,
            "wrap_egld should output only 1 payment"
        );

        EgldOrEsdtTokenPayment::from(returned_wrapped_egld.get(0))
    }

    fn unwrap_egld(&self, payment: EgldOrEsdtTokenPayment) -> EgldOrEsdtTokenPayment {
        let wrap_egld_addr = self.wrap_egld_addr().get();

        let ((), back_transfers) = self
            .wrap_egld_proxy(wrap_egld_addr)
            .unwrap_egld()
            .with_esdt_transfer(payment.unwrap_esdt())
            .execute_on_dest_context_with_back_transfers();

        let returned_egld = back_transfers.total_egld_amount;

        EgldOrEsdtTokenPayment::new(EgldOrEsdtTokenIdentifier::egld(), 0, returned_egld)
    }

    #[proxy]
    fn wrap_egld_proxy(
        &self,
        sc_address: ManagedAddress,
    ) -> multiversx_wegld_swap_sc::Proxy<Self::Api>;

    #[storage_mapper("wrapEgldAddr")]
    fn wrap_egld_addr(&self) -> SingleValueMapper<ManagedAddress>;
}
