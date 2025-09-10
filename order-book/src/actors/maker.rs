use crate::storage::{
    common_storage::{Percent, MAX_PERCENT},
    order::{Order, OrderDuration, OrderId, DAY_IN_SECONDS, HOUR_IN_SECONDS, MINUTE_IN_SECONDS},
};

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait MakerModule:
    crate::storage::order::OrderModule + crate::events::EventsModule + crate::pause::PauseModule
{
    #[payable("*")]
    #[endpoint(createOrder)]
    fn create_order(
        &self,
        output_token: TokenIdentifier,
        min_total_output: BigUint,
        order_duration: OrderDuration,
        opt_executor_fee: OptionalValue<Percent>,
    ) -> OrderId {
        self.require_not_paused();

        require!(
            output_token.is_valid_esdt_identifier(),
            "Invalid ESDT specified for output token"
        );

        let executor_fee = match opt_executor_fee {
            OptionalValue::Some(fee) => fee,
            OptionalValue::None => 0,
        };
        require!(executor_fee <= MAX_PERCENT, "Invalid executor fee");

        let current_timestamp = self.blockchain().get_block_timestamp();
        let mut expiration_timestamp = current_timestamp;
        expiration_timestamp += match &order_duration {
            OrderDuration::Minutes(minutes) => *minutes as u64 * MINUTE_IN_SECONDS,
            OrderDuration::Hours(hours) => *hours as u64 * HOUR_IN_SECONDS,
            OrderDuration::Days(days) => *days as u64 * DAY_IN_SECONDS,
        };
        require!(
            current_timestamp < expiration_timestamp,
            "Invalid expiration timestamp"
        );

        let caller = self.blockchain().get_caller();
        let (token_id, amount) = self.call_value().single_fungible_esdt();

        let order_id = self.get_and_increment_next_order_id();
        let order = Order {
            maker: caller,
            input_token: token_id.clone(),
            output_token,
            initial_input_amount: amount.clone(),
            current_input_amount: amount.clone(),
            min_total_output,
            executor_fee,
            creation_timestamp: current_timestamp,
            expiration_timestamp,
        };
        self.orders(order_id).set(&order);

        self.emit_create_order_event(order_id, order_duration, order);

        order_id
    }

    #[endpoint(cancelOrder)]
    fn cancel_order(&self, order_id: OrderId) {
        self.require_not_paused();
        self.require_valid_order_id(order_id);

        let caller = self.blockchain().get_caller();
        let order = self.orders(order_id).take();
        require!(
            order.maker == caller,
            "Invalid order ID - not the original order creator"
        );

        self.send()
            .direct_esdt(&caller, &order.input_token, 0, &order.current_input_amount);

        self.emit_cancel_order_event(order_id);
    }
}
