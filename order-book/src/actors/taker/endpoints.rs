use crate::{
    actors::{
        executor::SwapStatus,
        taker::common_types::{
            ProcessP2pFillArgs, ProcessP2pFillResult, INVALID_TOKEN_SENT_ERR_MSG,
            SENT_TOO_FEW_TOKENS_ERR_MSG,
        },
    },
    storage::order::OrderId,
};

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait EndpointsModule:
    super::internal::InternalModule
    + super::views::ViewsModule
    + crate::external_sc_interactions::router::RouterActionsModule
    + crate::storage::order::OrderModule
    + crate::storage::common_storage::CommonStorageModule
    + crate::events::EventsModule
    + crate::pause::PauseModule
{
    #[payable("*")]
    #[endpoint(fillOrderP2PByBuyingInput)]
    fn fill_order_p2p_by_buying_input(&self, order_id: OrderId, tokens_to_buy: BigUint) {
        self.require_not_paused();
        self.require_valid_order_id(order_id);

        let mut order = self.orders(order_id).get();
        let payment = self.call_value().single_esdt();
        require!(
            payment.token_identifier == order.output_token,
            INVALID_TOKEN_SENT_ERR_MSG
        );
        require!(
            tokens_to_buy <= order.current_input_amount,
            "Buying too many tokens"
        );

        let min_maker_amount = self.calculate_min_maker_amount(
            &order.min_total_output,
            &order.initial_input_amount,
            &tokens_to_buy,
        );
        require!(
            payment.amount >= min_maker_amount,
            SENT_TOO_FEW_TOKENS_ERR_MSG
        );

        let taker = self.blockchain().get_caller();
        let result = self.process_p2p_fill(
            ProcessP2pFillArgs {
                maker: &order.maker,
                input_token: order.input_token.clone(),
                min_maker_amount,
                taker: &taker,
                tokens_to_buy: tokens_to_buy.clone(),
                taker_payment: &payment,
            },
            None,
        );
        require!(
            matches!(result, ProcessP2pFillResult::Success),
            SENT_TOO_FEW_TOKENS_ERR_MSG
        );

        self.update_order_and_fire_events(order_id, &mut order, tokens_to_buy);
    }

    /// args are pairs of (order_id and tokens_to_buy)
    #[payable("*")]
    #[endpoint(fillOrdersP2PBatchByBuyingInput)]
    fn fill_orders_p2p_batch_by_buying_input(
        &self,
        args: MultiValueEncoded<MultiValue2<OrderId, BigUint>>,
    ) -> MultiValueEncoded<SwapStatus> {
        self.require_not_paused();

        let payments = self.call_value().all_esdt_transfers().clone_value();
        require!(payments.len() == args.len(), "Invalid arguments");

        let taker = self.blockchain().get_caller();
        let mut statuses = MultiValueEncoded::new();
        for (arg, payment) in args.into_iter().zip(payments.iter()) {
            let (order_id, tokens_to_buy) = arg.into_tuple();
            let is_valid = self.validate_batch_input(order_id, &payment, &tokens_to_buy);
            if !is_valid {
                statuses.push(SwapStatus::InvalidInput);

                continue;
            }

            let mut order = self.orders(order_id).get();
            let min_maker_amount = self.calculate_min_maker_amount(
                &order.min_total_output,
                &order.initial_input_amount,
                &tokens_to_buy,
            );
            let result = self.process_p2p_fill(
                ProcessP2pFillArgs {
                    maker: &order.maker,
                    input_token: order.input_token.clone(),
                    min_maker_amount,
                    taker: &taker,
                    tokens_to_buy: tokens_to_buy.clone(),
                    taker_payment: &payment,
                },
                None,
            );
            if matches!(result, ProcessP2pFillResult::Fail) {
                statuses.push(SwapStatus::Fail);

                continue;
            }

            self.update_order_and_fire_events(order_id, &mut order, tokens_to_buy);

            statuses.push(SwapStatus::Success);
        }

        statuses
    }

    #[payable("*")]
    #[endpoint(fillOrderP2PBySellingOutput)]
    fn fill_order_p2p_by_selling_output(&self, order_id: OrderId) {
        self.require_not_paused();
        self.require_valid_order_id(order_id);

        let mut order = self.orders(order_id).get();
        let payment = self.call_value().single_esdt();
        require!(
            payment.token_identifier == order.output_token,
            INVALID_TOKEN_SENT_ERR_MSG
        );

        let taker = self.blockchain().get_caller();
        let result = self.get_tokens_bought_by_p2p_sell_output_internal(&order, &payment.amount);

        // refund surplus to taker
        self.send().direct_non_zero_esdt_payment(
            &taker,
            &EsdtTokenPayment::new(payment.token_identifier.clone(), 0, result.surplus.clone()),
        );

        let min_maker_amount = self.calculate_min_maker_amount(
            &order.min_total_output,
            &order.initial_input_amount,
            &result.tokens_bought,
        );
        let tokens_to_buy = result.tokens_bought.clone();
        let result = self.process_p2p_fill(
            ProcessP2pFillArgs {
                maker: &order.maker,
                input_token: order.input_token.clone(),
                min_maker_amount,
                taker: &taker,
                tokens_to_buy: result.tokens_bought.clone(),
                taker_payment: &EsdtTokenPayment::new(
                    payment.token_identifier.clone(),
                    0,
                    result.taker_token_amount.clone(),
                ),
            },
            Some(result),
        );
        require!(
            matches!(result, ProcessP2pFillResult::Success),
            SENT_TOO_FEW_TOKENS_ERR_MSG
        );

        self.update_order_and_fire_events(order_id, &mut order, tokens_to_buy);
    }
}
