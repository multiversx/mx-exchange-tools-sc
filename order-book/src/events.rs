use auto_farm::common::chain_info::CurrentChainInfo;

use crate::storage::order::{Order, OrderDuration, OrderId};

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[type_abi]
#[derive(TopEncode, TopDecode)]
pub struct CreateOrderEventData<M: ManagedTypeApi> {
    pub order: Order<M>,
    pub duration: OrderDuration,
    pub chain_info: CurrentChainInfo,
}

#[type_abi]
#[derive(TopEncode, TopDecode)]
pub struct OrderExecutedPartlyEventData<M: ManagedTypeApi> {
    pub part_filled: BigUint<M>,
    pub remaining_amount: BigUint<M>,
    pub chain_info: CurrentChainInfo,
}

#[type_abi]
#[derive(TopEncode, TopDecode)]
pub struct OrderExecutedFullyEventData<M: ManagedTypeApi> {
    pub part_filled: BigUint<M>,
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
            CreateOrderEventData {
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
        self.prune_expired_order_event(order_id, CurrentChainInfo::new::<Self::Api>());
    }

    #[inline]
    fn emit_order_executed_partly_event(
        &self,
        order_id: OrderId,
        part_filled: BigUint,
        remaining_amount: BigUint,
    ) {
        self.order_executed_partly_event(
            order_id,
            OrderExecutedPartlyEventData {
                part_filled,
                remaining_amount,
                chain_info: CurrentChainInfo::new::<Self::Api>(),
            },
        );
    }

    #[inline]
    fn emit_order_executed_fully_event(&self, order_id: OrderId, part_filled: BigUint) {
        self.order_executed_fully_event(
            order_id,
            OrderExecutedFullyEventData {
                part_filled,
                chain_info: CurrentChainInfo::new::<Self::Api>(),
            },
        );
    }

    #[event("pauseContract")]
    fn pause_event(&self);

    #[event("unpauseContract")]
    fn unpause_event(&self);

    #[event("createOrderEvent")]
    fn create_order_event(
        &self,
        #[indexed] order_id: OrderId,
        event_data: CreateOrderEventData<Self::Api>,
    );

    #[event("cancelOrderEvent")]
    fn cancel_order_event(
        &self,
        #[indexed] order_id: OrderId,
        current_chain_info: CurrentChainInfo,
    );

    #[event("pruneExpiredOrderEvent")]
    fn prune_expired_order_event(
        &self,
        #[indexed] order_id: OrderId,
        current_chain_info: CurrentChainInfo,
    );

    #[event("orderExecutedPartlyEvent")]
    fn order_executed_partly_event(
        &self,
        #[indexed] order_id: OrderId,
        event_data: OrderExecutedPartlyEventData<Self::Api>,
    );

    #[event("orderExecutedFullyEvent")]
    fn order_executed_fully_event(
        &self,
        #[indexed] order_id: OrderId,
        event_data: OrderExecutedFullyEventData<Self::Api>,
    );
}
