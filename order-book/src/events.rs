use auto_farm::common::chain_info::CurrentChainInfo;

use crate::storage::order::{Order, OrderDuration, OrderId};

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[derive(TypeAbi, TopEncode, TopDecode)]
pub struct CreateOrderEvent<M: ManagedTypeApi> {
    pub order: Order<M>,
    pub duration: OrderDuration,
    pub chain_info: CurrentChainInfo,
}

#[multiversx_sc::module]
pub trait EventsModule {
    #[inline]
    fn emit_create_order_event(
        &self,
        order_id: OrderId,
        duration: OrderDuration,
        order: Order<Self::Api>,
    ) {
        self.create_order_event(
            order_id,
            CreateOrderEvent {
                order,
                duration,
                chain_info: CurrentChainInfo::new::<Self::Api>(),
            },
        );
    }

    #[inline]
    fn emit_cancel_order_event(&self, order_id: OrderId) {
        self.cancel_order_event(order_id, CurrentChainInfo::new::<Self::Api>());
    }

    #[inline]
    fn emit_prune_expired_order_event(&self, order_id: OrderId) {
        self.prune_expired_order_event(order_id);
    }

    #[event("createOrderEvent")]
    fn create_order_event(
        &self,
        #[indexed] order_id: OrderId,
        event_data: CreateOrderEvent<Self::Api>,
    );

    #[event("cancelOrderEvent")]
    fn cancel_order_event(
        &self,
        #[indexed] order_id: OrderId,
        current_chain_info: CurrentChainInfo,
    );

    #[event("pruneExpiredOrderEvent")]
    fn prune_expired_order_event(&self, #[indexed] order_id: OrderId);
}
