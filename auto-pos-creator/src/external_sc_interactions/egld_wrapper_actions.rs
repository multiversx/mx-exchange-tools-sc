multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait EgldWrapperActionsModule {
    #[allow(deprecated)]
    fn call_wrap_egld(&self, egld_amount: BigUint) -> EsdtTokenPayment {
        let wrapper_sc_address = self.egld_wrapper_address().get();
        let ((), back_transfers) = self
            .egld_wrapper_proxy(wrapper_sc_address)
            .wrap_egld()
            .with_egld_transfer(egld_amount)
            .execute_on_dest_context_with_back_transfers();

        let returned_wrapped_egld = back_transfers.esdt_payments;
        require!(
            returned_wrapped_egld.len() == 1,
            "wrap_egld should output only 1 payment"
        );

        let output_payment = returned_wrapped_egld.get(0).clone();
        output_payment
    }

    #[storage_mapper("egldWrapperAddress")]
    fn egld_wrapper_address(&self) -> SingleValueMapper<ManagedAddress>;

    #[proxy]
    fn egld_wrapper_proxy(
        &self,
        sc_address: ManagedAddress,
    ) -> multiversx_wegld_swap_sc::Proxy<Self::Api>;
}
