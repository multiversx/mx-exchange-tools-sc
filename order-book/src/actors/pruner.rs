use crate::storage::{common_storage::MAX_PERCENT, order::OrderId};

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait PrunerModule:
    crate::storage::order::OrderModule
    + crate::storage::common_storage::CommonStorageModule
    + crate::events::EventsModule
    + crate::pause::PauseModule
{
    #[endpoint(pruneExpiredOrder)]
    fn prune_expired_order(&self, order_id: OrderId) {
        self.require_not_paused();
        self.require_valid_order_id(order_id);

        let order = self.orders(order_id).take();
        let current_time = self.blockchain().get_block_timestamp();
        require!(
            order.expiration_timestamp <= current_time,
            "Order not expired yet"
        );

        let pruner_fee_percent = self.pruning_fee().get();
        let pruner_fee_amount = &order.current_input_amount * pruner_fee_percent / MAX_PERCENT;
        let remaining_maker_amount = &order.current_input_amount - &pruner_fee_amount;

        let pruner = self.blockchain().get_caller();
        self.send().direct_non_zero_esdt_payment(
            &pruner,
            &EsdtTokenPayment::new(order.input_token.clone(), 0, pruner_fee_amount),
        );
        self.send().direct_non_zero_esdt_payment(
            &order.maker,
            &EsdtTokenPayment::new(order.input_token, 0, remaining_maker_amount),
        );

        self.emit_prune_expired_order_event(order_id);
    }
}
