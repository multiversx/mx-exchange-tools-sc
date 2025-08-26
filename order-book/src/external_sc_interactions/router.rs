use router::multi_pair_swap::{
    ProxyTrait as _, SWAP_TOKENS_FIXED_INPUT_FUNC_NAME, SWAP_TOKENS_FIXED_OUTPUT_FUNC_NAME,
};

use crate::{
    actors::executor::{RouterEndpointName, SwapOperationType},
    storage::order::Order,
};

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait RouterActionsModule: crate::storage::common_storage::CommonStorageModule {
    // TODO: use the new execute on dest which returns status after upgrade
    fn execute_swap(
        &self,
        order: &Order<Self::Api>,
        input_token_amount: &BigUint,
        swap_path: &ManagedVec<SwapOperationType<Self::Api>>,
    ) -> OptionalValue<EsdtTokenPayment> {
        let router_address = self.router_address().get();
        let router_args = self.convert_to_router_args(order, input_token_amount, swap_path);
        let mut returned_payments: ManagedVec<EsdtTokenPayment> = self
            .router_proxy(router_address)
            .multi_pair_swap(router_args)
            .single_esdt(&order.input_token, 0, input_token_amount)
            .execute_on_dest_context();

        require!(
            !returned_payments.is_empty(),
            "No payments received from router"
        );

        let last_payment_index = returned_payments.len() - 1;
        let last_payment = returned_payments.get(last_payment_index);
        returned_payments.remove(last_payment_index);

        if !returned_payments.is_empty() {
            self.send().direct_multi(&order.maker, &returned_payments);
        }

        OptionalValue::Some(last_payment)
    }

    fn convert_to_router_args(
        &self,
        order: &Order<Self::Api>,
        input_token_amount: &BigUint,
        swap_path: &ManagedVec<SwapOperationType<Self::Api>>,
    ) -> MultiValueEncoded<MultiValue4<ManagedAddress, ManagedBuffer, TokenIdentifier, BigUint>>
    {
        let last_index = swap_path.len() - 1;
        let mut swap_operations = MultiValueEncoded::new();
        for (i, single_path) in swap_path.iter().enumerate() {
            let endpoint_name = match single_path.endpoint_name {
                RouterEndpointName::FixedInput => SWAP_TOKENS_FIXED_INPUT_FUNC_NAME,
                RouterEndpointName::FixedOutput => SWAP_TOKENS_FIXED_OUTPUT_FUNC_NAME,
            };
            let min_output = if i != last_index {
                BigUint::from(1u32)
            } else {
                &order.min_total_output * &order.initial_input_amount / input_token_amount
            };

            swap_operations.push(
                (
                    single_path.pair_address,
                    ManagedBuffer::from(endpoint_name),
                    single_path.output_token_id,
                    min_output,
                )
                    .into(),
            );
        }

        swap_operations
    }

    #[proxy]
    fn router_proxy(&self, sc_address: ManagedAddress) -> router::Proxy<Self::Api>;
}
