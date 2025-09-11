use router::multi_pair_swap::{
    SWAP_TOKENS_FIXED_INPUT_FUNC_NAME, SWAP_TOKENS_FIXED_OUTPUT_FUNC_NAME,
};

use crate::{
    actors::executor::{RouterEndpointName, SwapOperationType},
    external_sc_interactions::proxies::router_proxy,
    storage::order::Order,
};

multiversx_sc::imports!();

pub type RouterArg<M> =
    MultiValue4<ManagedAddress<M>, ManagedBuffer<M>, TokenIdentifier<M>, BigUint<M>>;

#[multiversx_sc::module]
pub trait RouterActionsModule: crate::storage::common_storage::CommonStorageModule {
    fn execute_swap(
        &self,
        order: &Order<Self::Api>,
        input_token_amount: &BigUint,
        swap_path: &ManagedVec<SwapOperationType<Self::Api>>,
    ) -> Option<EsdtTokenPayment> {
        let router_address = self.router_address().get();
        let router_args = self.convert_to_router_args(order, input_token_amount, swap_path);
        let result = self
            .tx()
            .to(router_address)
            .typed(router_proxy::RouterProxy)
            .multi_pair_swap(router_args)
            .single_esdt(&order.input_token, 0, input_token_amount)
            .returns(ReturnsHandledOrError::new().returns(ReturnsResult))
            .sync_call_fallible();

        let mut returned_payments = match result {
            Result::Ok(returned_payments) => returned_payments,
            Result::Err(_) => return None,
        };
        require!(
            !returned_payments.is_empty(),
            "No payments received from router"
        );

        let last_payment_index = returned_payments.len() - 1;
        let last_payment = (returned_payments.get(last_payment_index)).clone();
        returned_payments.remove(last_payment_index);

        if !returned_payments.is_empty() {
            self.send().direct_multi(&order.maker, &returned_payments);
        }

        Some(last_payment)
    }

    fn convert_to_router_args(
        &self,
        order: &Order<Self::Api>,
        input_token_amount: &BigUint,
        swap_path: &ManagedVec<SwapOperationType<Self::Api>>,
    ) -> MultiValueEncoded<RouterArg<Self::Api>> {
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
                self.calculate_min_maker_amount(
                    &order.min_total_output,
                    &order.initial_input_amount,
                    input_token_amount,
                )
            };

            swap_operations.push(
                (
                    single_path.pair_address.clone(),
                    ManagedBuffer::from(endpoint_name),
                    single_path.output_token_id.clone(),
                    min_output,
                )
                    .into(),
            );
        }

        swap_operations
    }

    #[inline]
    fn calculate_min_maker_amount(
        &self,
        min_total_output: &BigUint,
        initial_input_amount: &BigUint,
        current_token_input_amount: &BigUint,
    ) -> BigUint {
        min_total_output * initial_input_amount / current_token_input_amount
    }
}
