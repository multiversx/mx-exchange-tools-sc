multiversx_sc::imports!();

type SwapOperationType<M> =
    MultiValue4<ManagedAddress<M>, ManagedBuffer<M>, TokenIdentifier<M>, BigUint<M>>;

use core::convert::TryFrom;

use router::{factory::ProxyTrait as _, multi_pair_swap::ProxyTrait as _};

#[multiversx_sc::module]
pub trait RouterActionsModule {
    fn multi_pair_swap(
        &self,
        start_payment: EsdtTokenPayment<Self::Api>,
        swap_args: ManagedVec<ManagedBuffer<Self::Api>>,
    ) -> ManagedVec<EsdtTokenPayment> {
        let router_addr = self.router_addr().get();

        let mut swap_operations = MultiValueEncoded::new();
        let mut swap_args_iter = swap_args.into_iter();
        let mut last_payment = EgldOrEsdtTokenPayment::from(start_payment.clone());

        loop {
            let pair_address_arg = match swap_args_iter.next() {
                Some(addr) => ManagedAddress::try_from(addr).unwrap_or_else(|err| sc_panic!(err)),
                None => break,
            };
            let function_wanted = match swap_args_iter.next() {
                Some(function) => function,
                None => break,
            };
            let token_wanted = match swap_args_iter.next() {
                Some(token) => TokenIdentifier::from(token),
                None => break,
            };
            let amount_wanted = match swap_args_iter.next() {
                Some(amount) => BigUint::from(amount),
                None => break,
            };
            swap_operations.push(SwapOperationType::from((
                pair_address_arg,
                function_wanted,
                token_wanted.clone(),
                amount_wanted.clone(),
            )));
            last_payment.token_identifier = EgldOrEsdtTokenIdentifier::esdt(token_wanted);
            last_payment.amount = amount_wanted;
        }

        let ((), back_transfers) = self
            .router_proxy(router_addr)
            .multi_pair_swap(swap_operations)
            .with_esdt_transfer(start_payment)
            .execute_on_dest_context_with_back_transfers();

        back_transfers.esdt_payments
    }

    #[view(getPair)]
    fn get_pair(
        &self,
        first_token_id: TokenIdentifier,
        second_token_id: TokenIdentifier,
    ) -> ManagedAddress {
        let router_addr = self.router_addr().get();

        self.router_proxy(router_addr)
            .get_pair(first_token_id, second_token_id)
            .execute_on_dest_context()
    }

    #[proxy]
    fn router_proxy(&self, sc_address: ManagedAddress) -> router::Proxy<Self::Api>;

    #[storage_mapper("routerAddr")]
    fn router_addr(&self) -> SingleValueMapper<ManagedAddress>;
}
