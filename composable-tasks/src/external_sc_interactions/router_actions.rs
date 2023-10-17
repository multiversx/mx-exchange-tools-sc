multiversx_sc::imports!();

type SwapOperationType<M> =
    MultiValue4<ManagedAddress<M>, ManagedBuffer<M>, TokenIdentifier<M>, BigUint<M>>;

use core::convert::TryFrom;

use router::multi_pair_swap::ProxyTrait as _;
pub const SWAP_TOKENS_FIXED_INPUT_FUNC_NAME: &[u8] = b"swapTokensFixedInput";

#[multiversx_sc::module]
pub trait RouterActionsModule {
    fn multi_pair_swap(
        &self,
        start_payment: EsdtTokenPayment<Self::Api>,
        swap_args: ManagedVec<ManagedBuffer<Self::Api>>,
    ) -> EgldOrEsdtTokenPayment {
        let router_addr = self.router_addr().get();

        let mut swap_operations = MultiValueEncoded::new();
        let mut swap_args_iter = swap_args.into_iter();
        let mut last_payment = EgldOrEsdtTokenPayment::from(start_payment.clone());

        loop {
            let pair_address_arg = match swap_args_iter.next() {
                Some(addr) => ManagedAddress::try_from(addr).unwrap_or_else(|err| sc_panic!(err)),
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
                ManagedBuffer::from(SWAP_TOKENS_FIXED_INPUT_FUNC_NAME),
                token_wanted.clone(),
                amount_wanted.clone(),
            )));
            last_payment.token_identifier = EgldOrEsdtTokenIdentifier::esdt(token_wanted);
            last_payment.amount = amount_wanted;
        }

        let resulted_payments: ManagedVec<EsdtTokenPayment> = self
            .router_proxy(router_addr)
            .multi_pair_swap(swap_operations)
            .with_esdt_transfer(start_payment)
            .execute_on_dest_context();

        require!(
            resulted_payments.len() == 1,
            "Router should output only 1 payment"
        );
        EgldOrEsdtTokenPayment::from(resulted_payments.get(0))
    }

    #[proxy]
    fn router_proxy(&self, sc_address: ManagedAddress) -> router::Proxy<Self::Api>;

    #[storage_mapper("routerAddr")]
    fn router_addr(&self) -> SingleValueMapper<ManagedAddress>;
}
