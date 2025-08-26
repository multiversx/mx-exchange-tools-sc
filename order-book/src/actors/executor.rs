use router::multi_pair_swap::{
    ProxyTrait as _, SWAP_TOKENS_FIXED_INPUT_FUNC_NAME, SWAP_TOKENS_FIXED_OUTPUT_FUNC_NAME,
};

use crate::storage::order::{Order, OrderId};

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[derive(TypeAbi, TopEncode, TopDecode)]
pub enum SwapStatus {
    InvalidInput,
    Fail,
    Success,
}

#[derive(TypeAbi, TopEncode, TopDecode, NestedEncode, NestedDecode, ManagedVecItem)]
pub enum RouterEndpointName {
    FixedInput,
    FixedOutput,
}

#[derive(TypeAbi, TopEncode, TopDecode, NestedEncode, NestedDecode, ManagedVecItem)]
pub struct SwapOperationType<M: ManagedTypeApi> {
    pub pair_address: ManagedAddress<M>,
    pub endpoint_name: RouterEndpointName,
    pub output_token_id: TokenIdentifier<M>,
}

#[multiversx_sc::module]
pub trait ExecutorModule:
    crate::storage::order::OrderModule
    + crate::storage::common_storage::CommonStorageModule
    + crate::events::EventsModule
    + crate::pause::PauseModule
{
    /// args are pairs of (order ID, token amount to swap, vec of SwapOperationType - which will be passed to router when swapping)
    #[endpoint(executeOrders)]
    fn execute_orders(
        &self,
        args: MultiValueEncoded<
            MultiValue3<OrderId, BigUint, ManagedVec<SwapOperationType<Self::Api>>>,
        >,
    ) -> MultiValueEncoded<SwapStatus> {
        let caller = self.blockchain().get_caller();
        require!(
            self.executor_whitelist().contains(&caller),
            "Not in executor whitelist"
        );

        let mut swap_statuses = MultiValueEncoded::new();
        for arg in args {
            let (order_id, input_token_amount, swap_path) = arg.into_tuple();
            let is_valid = self.validate_input(order_id, &input_token_amount, &swap_path);
            if !is_valid {
                swap_statuses.push(SwapStatus::InvalidInput);

                continue;
            }

            let order = self.orders(order_id).get();
            let opt_tokens_out = self.execute_swap(&order, &input_token_amount, &swap_path);
            match opt_tokens_out {
                OptionalValue::Some(payment) => {
                    self.update_order_after_success(order_id, order, &payment, &input_token_amount);
                    self.distribute_tokens();

                    swap_statuses.push(SwapStatus::Success);
                }
                OptionalValue::None => {
                    swap_statuses.push(SwapStatus::Fail);
                }
            }
        }

        swap_statuses
    }

    /// returns `true` if input is valid, `false` otherwise
    #[must_use]
    fn validate_input(
        &self,
        order_id: OrderId,
        token_amount: &BigUint,
        swap_path: &ManagedVec<SwapOperationType<Self::Api>>,
    ) -> bool {
        if token_amount == &0 {
            return false;
        }

        let order_mapper = self.orders(order_id);
        if order_mapper.is_empty() {
            return false;
        }

        let order = order_mapper.get();
        let current_time = self.blockchain().get_block_timestamp();
        if order.expiration_timestamp >= current_time {
            return false;
        }
        if token_amount > &order.current_input_amount {
            return false;
        }

        if swap_path.is_empty() {
            return false;
        }

        let last_item_index = swap_path.len() - 1;
        let last_swap_path = swap_path.get(last_item_index);
        if last_swap_path.output_token_id != order.output_token {
            return false;
        }

        true
    }

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

    fn update_order_after_success(
        &self,
        order_id: OrderId,
        mut order: Order<Self::Api>,
        received_payment: &EsdtTokenPayment,
        input_token_amount: &BigUint,
    ) {
        require!(
            received_payment.token_identifier == order.output_token,
            "Invalid token received from router"
        );

        if order.current_input_amount > 0 {
            order.current_input_amount -= input_token_amount;
            self.orders(order_id).set(order);

            // TODO: event
        } else {
            self.orders(order_id).clear();

            // TODO: event
        }
    }

    fn distribute_tokens(&self) {}

    #[proxy]
    fn router_proxy(&self, sc_address: ManagedAddress) -> router::Proxy<Self::Api>;
}
