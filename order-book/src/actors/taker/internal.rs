use crate::{
    actors::taker::common_types::{
        GetTokensBoughtP2pResultType, ProcessP2pFillArgs, ProcessP2pFillResult,
    },
    storage::{
        common_storage::MAX_PERCENT,
        order::{Order, OrderId},
    },
};

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait InternalModule:
    crate::external_sc_interactions::router::RouterActionsModule
    + crate::storage::order::OrderModule
    + crate::storage::common_storage::CommonStorageModule
    + crate::events::EventsModule
{
    /// returns `true` if input is valid, `false` otherwise
    #[must_use]
    fn validate_batch_input(
        &self,
        order_id: OrderId,
        payment: &EsdtTokenPayment,
        tokens_to_buy: &BigUint,
    ) -> bool {
        let order_mapper = self.orders(order_id);
        if order_mapper.is_empty() {
            return false;
        }

        let order = order_mapper.get();
        if payment.token_identifier != order.output_token {
            return false;
        }
        if tokens_to_buy > &order.current_input_amount {
            return false;
        }

        let min_maker_amount = self.calculate_min_maker_amount(
            &order.min_total_output,
            &order.initial_input_amount,
            tokens_to_buy,
        );
        if payment.amount < min_maker_amount {
            return false;
        }

        true
    }

    fn get_tokens_bought_by_p2p_sell_output_internal(
        &self,
        order: &Order<Self::Api>,
        payment_amount: &BigUint,
    ) -> GetTokensBoughtP2pResultType<Self::Api> {
        let protocol_fee_percent = self.p2p_protocol_fee().get();
        let max_payment_amount =
            &order.current_input_amount * &order.min_total_output / &order.initial_input_amount;
        let max_fee = &max_payment_amount * protocol_fee_percent / MAX_PERCENT;
        let max_amount_with_fee = &max_payment_amount + &max_fee;

        if payment_amount < &max_amount_with_fee {
            let total_protocol_fee = payment_amount * protocol_fee_percent / MAX_PERCENT;
            let remaining_tokens_taker = payment_amount - &total_protocol_fee;

            GetTokensBoughtP2pResultType {
                tokens_bought: &remaining_tokens_taker * &order.initial_input_amount
                    / &order.min_total_output,
                taker_token_amount: remaining_tokens_taker,
                fees: total_protocol_fee,
                surplus: BigUint::zero(),
            }
        } else {
            let surplus = payment_amount - &max_amount_with_fee;

            GetTokensBoughtP2pResultType {
                tokens_bought: order.current_input_amount.clone(),
                taker_token_amount: max_payment_amount,
                fees: max_fee,
                surplus,
            }
        }
    }

    #[must_use]
    fn process_p2p_fill(
        &self,
        args: ProcessP2pFillArgs<Self::Api>,
        opt_sell_p2p_arg: Option<GetTokensBoughtP2pResultType<Self::Api>>,
    ) -> ProcessP2pFillResult {
        let (remaining_tokens_maker, total_protocol_fee) = match opt_sell_p2p_arg {
            Some(sell_p2p_arg) => (sell_p2p_arg.taker_token_amount, sell_p2p_arg.fees),
            None => {
                let protocol_fee_percent = self.p2p_protocol_fee().get();
                let total_protocol_fee =
                    &args.taker_payment.amount * protocol_fee_percent / MAX_PERCENT;
                let remaining_tokens_maker = &args.taker_payment.amount - &total_protocol_fee;

                (remaining_tokens_maker, total_protocol_fee)
            }
        };

        if remaining_tokens_maker < args.min_maker_amount {
            return ProcessP2pFillResult::Fail;
        }

        let surplus = &remaining_tokens_maker - &args.min_maker_amount;
        let total_treasury_amount = total_protocol_fee + surplus;
        let treasury_addresss = self.treasury_address().get();
        self.send().direct_non_zero_esdt_payment(
            &treasury_addresss,
            &EsdtTokenPayment::new(
                args.taker_payment.token_identifier.clone(),
                0,
                total_treasury_amount,
            ),
        );

        self.send().direct_non_zero_esdt_payment(
            args.maker,
            &EsdtTokenPayment::new(
                args.taker_payment.token_identifier.clone(),
                0,
                args.min_maker_amount,
            ),
        );
        self.send().direct_non_zero_esdt_payment(
            args.taker,
            &EsdtTokenPayment::new(args.input_token, 0, args.tokens_to_buy),
        );

        ProcessP2pFillResult::Success
    }
}
