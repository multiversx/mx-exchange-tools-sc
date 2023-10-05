multiversx_sc::imports!();

use multiversx_wegld_swap_sc::ProxyTrait as _;

#[multiversx_sc::module]
pub trait WegldSwapModule {

    fn wrap_egld(
        &self,
    ) -> EgldOrEsdtTokenPayment {
        let egld_payment = self.call_value().egld_value();
        let wrap_egld_addr = self.wrap_egld_addr().get();

        let wrapped_egld: EsdtTokenPayment = self.wrap_egld_proxy(wrap_egld_addr)
            .wrap_egld()
            .with_egld_transfer(egld_payment.clone_value())
            .execute_on_dest_context();

        EgldOrEsdtTokenPayment::from(wrapped_egld)
    }

    fn unwrap_egld(
        &self,
    ) -> EgldOrEsdtTokenPayment {
        let wrap_egld_payment = self.call_value().single_esdt();
        let wrap_egld_addr = self.wrap_egld_addr().get();

        let _: IgnoreValue = self.wrap_egld_proxy(wrap_egld_addr)
            .unwrap_egld()
            .with_esdt_transfer(wrap_egld_payment.clone())
            .execute_on_dest_context();
        EgldOrEsdtTokenPayment::new(EgldOrEsdtTokenIdentifier::egld(), 0, wrap_egld_payment.amount)
    }

    
    #[proxy]
    fn wrap_egld_proxy(&self, sc_address: ManagedAddress) -> multiversx_wegld_swap_sc::Proxy<Self::Api>;

    #[storage_mapper("wrapEgldAddr")]
    fn wrap_egld_addr(&self) -> SingleValueMapper<ManagedAddress>;
}
