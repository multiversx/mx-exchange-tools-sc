multiversx_sc::imports!();
multiversx_sc::derive_imports!();

pub type OrderId = u64;
pub type Timestamp = u64;

#[derive(TypeAbi, TopEncode, TopDecode, NestedEncode, NestedDecode)]
pub enum OrderStatus {
    Pending,
    PartiallyFilled,
    Executed,
    Cancelled,
    Expired,
}

/// `min_exchange_rate`: For example, if you want a 1:2 exchange rate, this should be 10^18 * 2 (considering token B has 18 decimals)
#[derive(TypeAbi, TopEncode, TopDecode)]
pub struct Order<M: ManagedTypeApi> {
    pub makers_id: AddressId,
    pub input_token: TokenIdentifier<M>,
    pub output_token: TokenIdentifier<M>,
    pub initial_input_amount: BigUint<M>,
    pub current_input_amount: BigUint<M>,
    pub min_exchange_rate: BigUint<M>,
    pub executor_fee: BigUint<M>, // TODO: Maybe remove. Send to executor directly
    pub order_status: OrderStatus,
    pub creation_timestamp: Timestamp,
    pub expiration_timestamp: Timestamp, // TODO: Make this easier to give from user perspective by creating an enum with Minute, Hour, Day, Month
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

    fn get_and_increment_next_order_id(&self) -> OrderId {
        self.next_order_id().update(|next_order_id| {
            let to_return = *next_order_id;
            *next_order_id += 1;

            to_return
        })
    }

    #[view(getOrderInfo)]
    #[storage_mapper("orders")]
    fn orders(&self, order_id: OrderId) -> SingleValueMapper<Order<Self::Api>>;

    #[storage_mapper("nextOrderId")]
    fn next_order_id(&self) -> SingleValueMapper<OrderId>;
}
