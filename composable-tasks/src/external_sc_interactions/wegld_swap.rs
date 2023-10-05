multiversx_sc::imports!();

use multiversx_wegld_swap_sc::ProxyTrait as _;

#[multiversx_sc::module]
pub trait WegldSwapModule {

    fn wrap_egld(
        &self,
    ) -> EsdtTokenPayment {
        let egld_payment = self.call_value().egld_value();
        let wrap_egld_addr = self.wrap_egld_addr().get();

        self.wrap_egld_proxy(wrap_egld_addr)
            .wrap_egld()
            .with_egld_transfer(egld_payment.clone_value())
            .execute_on_dest_context()
    }

    fn unwrap_egld(
        &self,
    ) -> EsdtTokenPayment {
        let wrap_egld_payment = self.call_value().single_esdt();
        let wrap_egld_addr = self.wrap_egld_addr().get();

        self.wrap_egld_proxy(wrap_egld_addr)
            .unwrap_egld()
            .with_esdt_transfer(wrap_egld_payment)
            .execute_on_dest_context()
    }

    
    #[proxy]
    fn wrap_egld_proxy(&self, sc_address: ManagedAddress) -> multiversx_wegld_swap_sc::Proxy<Self::Api>;

    #[storage_mapper("wrapEgldAddr")]
    fn wrap_egld_addr(&self) -> SingleValueMapper<ManagedAddress>;
}
