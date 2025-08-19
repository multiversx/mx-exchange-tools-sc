use auto_farm::common::chain_info::CurrentChainInfo;

use crate::storage::order::{Order, OrderId};

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[derive(TypeAbi, TopEncode)]
pub struct CreateOrderEvent<'a, M: ManagedTypeApi> {
    pub caller: &'a ManagedAddress<M>,
    pub order_id: OrderId,
    pub order: &'a Order<M>,
    pub chain_info: CurrentChainInfo,
}

#[multiversx_sc::module]
pub trait EventsModule {
    #[inline]
    fn emit_create_order_event(&self, order_id: OrderId, order: &Order<Self::Api>) {
        self.create_order_event(order_id, order, CurrentChainInfo::new::<Self::Api>());
    }

    #[inline]
    fn emit_cancel_order_event(&self, order_id: OrderId) {
        self.cancel_order_event(order_id, CurrentChainInfo::new::<Self::Api>());
    }

    #[event("createOrderEvent")]
    fn create_order_event(
        &self,
        #[indexed] order_id: OrderId,
        #[indexed] order: &Order<Self::Api>,
        current_chain_info: CurrentChainInfo,
    );

    #[event("cancelOrderEvent")]
    fn cancel_order_event(
        &self,
        #[indexed] order_id: OrderId,
        current_chain_info: CurrentChainInfo,
    );
}
