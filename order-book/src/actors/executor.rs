use crate::storage::order::{Order, OrderId};

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[derive(TypeAbi, TopEncode, TopDecode)]
pub enum SwapStatus {
    Success,
    Fail,
}

#[multiversx_sc::module]
pub trait ExecutorModule:
    crate::storage::order::OrderModule
    + crate::storage::common_storage::CommonStorageModule
    + crate::events::EventsModule
    + crate::pause::PauseModule
{
    #[endpoint(executeOrders)]
    fn execute_orders(
        &self,
        order_id_token_amount_pairs: MultiValueEncoded<MultiValue2<OrderId, BigUint>>,
    ) -> MultiValueEncoded<SwapStatus> {
        let mut swap_statuses = MultiValueEncoded::new();

        for pair in order_id_token_amount_pairs {
            let (order_id, token_amount) = pair.into_tuple();
            let opt_order = self.validate_input_and_get_order(order_id, &token_amount);
            if opt_order.is_none() {
                swap_statuses.push(SwapStatus::Fail);

                continue;
            }

            let opt_tokens_out = self.execute_swap();
            match opt_tokens_out {
                OptionalValue::Some(payment) => {
                    let mut order = unsafe { opt_order.unwrap_unchecked() };
                    require!(
                        payment.token_identifier == order.output_token,
                        "Invalid token received from router"
                    );

                    order.current_input_amount -= token_amount;
                    self.orders(order_id).set(order);

                    swap_statuses.push(SwapStatus::Success);

                    // TODO: Clear order if all filled
                    // TODO: Distribute tokens
                }
                OptionalValue::None => {
                    swap_statuses.push(SwapStatus::Fail);
                }
            }
        }

        // TODO: event

        swap_statuses
    }

    fn validate_input_and_get_order(
        &self,
        order_id: OrderId,
        token_amount: &BigUint,
    ) -> Option<Order<Self::Api>> {
        if token_amount == &0 {
            return None;
        }

        let order_mapper = self.orders(order_id);
        if order_mapper.is_empty() {
            return None;
        }

        let order = order_mapper.get();
        let current_time = self.blockchain().get_block_timestamp();
        if order.expiration_timestamp >= current_time {
            return None;
        }
        if token_amount > &order.current_input_amount {
            return None;
        }

        Some(order)
    }

    // TODO: use the new execute on dest which returns status after upgrade
    fn execute_swap(&self) -> OptionalValue<EsdtTokenPayment> {
        // TODO: Calculate min output token amount, pass arg to router

        OptionalValue::None
    }
}
