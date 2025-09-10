use crate::storage::{
    common_storage::MAX_PERCENT,
    order::{Order, OrderId},
};

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[type_abi]
#[derive(TopEncode, TopDecode)]
pub enum SwapStatus {
    InvalidInput,
    Fail,
    Success,
}

#[type_abi]
#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, ManagedVecItem)]
pub enum RouterEndpointName {
    FixedInput,
    FixedOutput,
}

#[type_abi]
#[derive(TopEncode, TopDecode, NestedEncode, NestedDecode, ManagedVecItem)]
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
        self.require_not_paused();

        let executor = self.get_executor();
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
                Some(payment) => {
                    require!(
                        payment.token_identifier == order.output_token,
                        "Invalid token received from router"
                    );

                    self.update_order_and_fire_events(
                        order_id,
                        &mut order,
                        input_token_amount.clone(),
                    );
                    self.distribute_tokens(&order, &executor, &input_token_amount, &payment);

                    swap_statuses.push(SwapStatus::Success);
                }
                None => {
                    swap_statuses.push(SwapStatus::Fail);
                }
            }
        }

        swap_statuses
    }

    fn get_executor(&self) -> ManagedAddress {
        let executor = self.blockchain().get_caller();
        require!(
            self.executor_whitelist().contains(&executor),
            "Not in executor whitelist"
        );

        executor
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
        if order.expiration_timestamp < current_time {
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

    fn distribute_tokens(
        &self,
        order: &Order<Self::Api>,
        executor: &ManagedAddress,
        input_token_amount: &BigUint,
        output_tokens: &EsdtTokenPayment,
    ) {
        let min_maker_amount = self.calculate_min_maker_amount(
            &order.min_total_output,
            &order.initial_input_amount,
            input_token_amount,
        );

        let mut total_executor_amount = &output_tokens.amount * order.executor_fee / MAX_PERCENT;
        let mut maker_amount = &output_tokens.amount - &total_executor_amount;
        if maker_amount > min_maker_amount {
            let surplus = &maker_amount - &min_maker_amount;
            total_executor_amount += &surplus;
            maker_amount -= surplus;
        }

        self.send().direct_non_zero_esdt_payment(
            &order.maker,
            &EsdtTokenPayment::new(output_tokens.token_identifier.clone(), 0, maker_amount),
        );
        self.send().direct_non_zero_esdt_payment(
            executor,
            &EsdtTokenPayment::new(
                output_tokens.token_identifier.clone(),
                0,
                total_executor_amount,
            ),
        );
    }
}
