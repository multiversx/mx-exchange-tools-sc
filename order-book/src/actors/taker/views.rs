use crate::{
    actors::taker::common_types::GetTokensBoughtP2pResultType,
    storage::{common_storage::MAX_PERCENT, order::OrderId},
};

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait ViewsModule:
    super::internal::InternalModule
    + crate::external_sc_interactions::router::RouterActionsModule
    + crate::storage::order::OrderModule
    + crate::storage::common_storage::CommonStorageModule
    + crate::events::EventsModule
{
    #[view(getTokensNeededForP2pBuyInput)]
    fn get_tokens_needed_for_p2p_buy_input(
        &self,
        order_id: OrderId,
        tokens_to_buy: BigUint,
    ) -> BigUint {
        let order = self.orders(order_id).get();
        let protocol_fee_percent = self.p2p_protocol_fee().get();
        let payment_amount_needed =
            tokens_to_buy * &order.min_total_output / &order.initial_input_amount;

        payment_amount_needed * MAX_PERCENT / (MAX_PERCENT - protocol_fee_percent)
    }

    #[view(getTokensBoughtByP2pSellOutput)]
    fn get_tokens_bought_by_p2p_sell_output(
        &self,
        order_id: OrderId,
        payment_amount: BigUint,
    ) -> GetTokensBoughtP2pResultType<Self::Api> {
        let order = self.orders(order_id).get();
        self.get_tokens_bought_by_p2p_sell_output_internal(&order, &payment_amount)
    }
}
