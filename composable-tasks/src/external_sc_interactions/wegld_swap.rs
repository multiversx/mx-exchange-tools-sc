multiversx_sc::imports!();

use multiversx_wegld_swap_sc::ProxyTrait as _;

#[multiversx_sc::module]
pub trait WegldWrapModule {
    fn wrap_egld(&self, payment: EgldOrEsdtTokenPayment) -> EgldOrEsdtTokenPayment {
        let wrap_egld_addr = self.wrap_egld_addr().get();
        require!(
            payment.token_identifier.is_egld(),
            "Payment token is not EGLD!"
        );

        let wrapped_egld: EsdtTokenPayment = self
            .wrap_egld_proxy(wrap_egld_addr)
            .wrap_egld()
            .with_egld_transfer(payment.amount)
            .execute_on_dest_context();

        EgldOrEsdtTokenPayment::from(wrapped_egld)
    }

    fn unwrap_egld(&self, payment: EgldOrEsdtTokenPayment) -> EgldOrEsdtTokenPayment {
        let wrap_egld_addr = self.wrap_egld_addr().get();

        let _: IgnoreValue = self
            .wrap_egld_proxy(wrap_egld_addr)
            .unwrap_egld()
            .with_esdt_transfer(payment.clone().unwrap_esdt())
            .execute_on_dest_context();
        EgldOrEsdtTokenPayment::new(EgldOrEsdtTokenIdentifier::egld(), 0, payment.amount)
    }

    #[proxy]
    fn wrap_egld_proxy(
        &self,
        sc_address: ManagedAddress,
    ) -> multiversx_wegld_swap_sc::Proxy<Self::Api>;

    #[storage_mapper("wrapEgldAddr")]
    fn wrap_egld_addr(&self) -> SingleValueMapper<ManagedAddress>;
}
