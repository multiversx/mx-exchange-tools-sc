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
    crate::external_sc_interactions::router::RouterActionsModule
    + crate::storage::order::OrderModule
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

            let mut order = self.orders(order_id).get();
            let opt_tokens_out = self.execute_swap(&order, &input_token_amount, &swap_path);
            match opt_tokens_out {
                OptionalValue::Some(payment) => {
                    self.update_order_after_success(
                        order_id,
                        &mut order,
                        &payment,
                        &input_token_amount,
                    );
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

    fn update_order_after_success(
        &self,
        order_id: OrderId,
        order: &mut Order<Self::Api>,
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
}
