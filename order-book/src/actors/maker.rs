use crate::storage::{
    common_storage::{Percent, MAX_PERCENT},
    order::{
        Order, OrderDuration, OrderId, OrderStatus, DAY_IN_SECONDS, HOUR_IN_SECONDS,
        MINUTE_IN_SECONDS,
    },
};

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait MakerModule: crate::storage::order::OrderModule + crate::events::EventsModule {
    #[payable("*")]
    #[endpoint(createOrder)]
    fn create_order(
        &self,
        output_token: TokenIdentifier,
        min_total_output: BigUint,
        order_duration: OrderDuration,
        opt_executor_fee: OptionalValue<Percent>,
    ) -> OrderId {
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
        let expiration_timestamp = match order_duration {
            OrderDuration::Minutes(minutes) => minutes as u64 * MINUTE_IN_SECONDS,
            OrderDuration::Hours(hours) => hours as u64 * HOUR_IN_SECONDS,
            OrderDuration::Days(days) => days as u64 * DAY_IN_SECONDS,
        };
        require!(
            current_timestamp < expiration_timestamp,
            "Invalid expiration timestamp"
        );

        let caller = self.blockchain().get_caller();
        let maker_id = self.maker_id().get_id_or_insert(&caller);
        let (token_id, amount) = self.call_value().single_fungible_esdt();

        let order_id = self.get_and_increment_next_order_id();
        let order = Order {
            maker_id,
            input_token: token_id,
            output_token,
            initial_input_amount: amount.clone(),
            current_input_amount: amount,
            min_total_output,
            executor_fee,
            status: OrderStatus::Pending,
            creation_timestamp: current_timestamp,
            expiration_timestamp,
        };
        self.orders(order_id).set(order);

        // TODO: Event

        order_id
    }

    #[storage_mapper("makerId")]
    fn maker_id(&self) -> AddressToIdMapper;
}
