use crate::storage::common_storage::Percent;

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

pub type OrderId = u64;
pub type Timestamp = u64;

pub const MINUTE_IN_SECONDS: Timestamp = 60;
pub const HOUR_IN_SECONDS: Timestamp = 60 * MINUTE_IN_SECONDS;
pub const DAY_IN_SECONDS: Timestamp = 24 * HOUR_IN_SECONDS;

#[derive(TypeAbi, TopEncode, TopDecode, NestedEncode, NestedDecode)]
pub enum OrderDuration {
    Minutes(u8),
    Hours(u8),
    Days(u8),
}

#[derive(TypeAbi, TopEncode, TopDecode, NestedEncode, NestedDecode)]
pub struct Order<M: ManagedTypeApi> {
    pub maker: ManagedAddress<M>,
    pub input_token: TokenIdentifier<M>,
    pub output_token: TokenIdentifier<M>,
    pub initial_input_amount: BigUint<M>,
    pub current_input_amount: BigUint<M>,
    pub min_total_output: BigUint<M>,
    pub executor_fee: Percent,
    pub creation_timestamp: Timestamp,
    pub expiration_timestamp: Timestamp,
}

#[multiversx_sc::module]
pub trait OrderModule {
    #[view(getLastOrderId)]
    fn get_last_order_id(&self) -> OptionalValue<OrderId> {
        let next_order_id = self.next_order_id().get();
        if next_order_id != 0 {
            OptionalValue::Some(next_order_id - 1)
        } else {
            OptionalValue::None
        }
    }

    #[view(getOrders)]
    fn get_orders(
        &self,
        start_id: OrderId,
        return_data_limit: usize,
    ) -> MultiValueEncoded<MultiValue2<OrderId, Order<Self::Api>>> {
        let mut result = MultiValueEncoded::new();

        let next_order_id = self.next_order_id().get();
        if start_id >= next_order_id {
            return result;
        }

        let mut result_len = 0;
        for current_id in start_id..next_order_id {
            let order_mapper = self.orders(current_id);
            if order_mapper.is_empty() {
                continue;
            }

            let order = order_mapper.get();
            result.push((current_id, order).into());
            result_len += 1;

            if result_len == return_data_limit {
                break;
            }
        }

        result
    }

    fn get_and_increment_next_order_id(&self) -> OrderId {
        self.next_order_id().update(|next_order_id| {
            let to_return = *next_order_id;
            *next_order_id += 1;

            to_return
        })
    }

    fn require_valid_order_id(&self, order_id: OrderId) {
        require!(
            !self.orders(order_id).is_empty(),
            "Order doesn't exist or executed/expired/cancelled already"
        );
    }

    #[view(getOrderInfo)]
    #[storage_mapper("orders")]
    fn orders(&self, order_id: OrderId) -> SingleValueMapper<Order<Self::Api>>;

    #[storage_mapper("nextOrderId")]
    fn next_order_id(&self) -> SingleValueMapper<OrderId>;
}
