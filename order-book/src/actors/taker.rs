use crate::{
    actors::executor::SwapStatus,
    storage::{
        common_storage::MAX_PERCENT,
        order::{Order, OrderId},
    },
};

multiversx_sc::imports!();

pub static INVALID_TOKEN_SENT_ERR_MSG: &[u8] = b"Invalid token sent";
pub static SENT_TOO_FEW_TOKENS_ERR_MSG: &[u8] = b"Sent too few tokens";

pub struct ProcessP2pFillArgs<'a, M: ManagedTypeApi> {
    pub maker: &'a ManagedAddress<M>,
    pub input_token: TokenIdentifier<M>,
    pub min_maker_amount: BigUint<M>,
    pub taker: &'a ManagedAddress<M>,
    pub tokens_to_buy: BigUint<M>,
    pub taker_payment: &'a EsdtTokenPayment<M>,
}

pub enum ProcessP2pFillResult {
    Fail,
    Success,
}

#[multiversx_sc::module]
pub trait TakerModule:
    crate::external_sc_interactions::router::RouterActionsModule
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
        let result = self.process_p2p_fill(ProcessP2pFillArgs {
            maker: &order.maker,
            input_token: order.input_token.clone(),
            min_maker_amount,
            taker: &taker,
            tokens_to_buy: tokens_to_buy.clone(),
            taker_payment: &payment,
        });
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
            let result = self.process_p2p_fill(ProcessP2pFillArgs {
                maker: &order.maker,
                input_token: order.input_token.clone(),
                min_maker_amount,
                taker: &taker,
                tokens_to_buy: tokens_to_buy.clone(),
                taker_payment: &payment,
            });
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
        let tokens_to_buy = self.get_tokens_to_buy_and_refund_surplus(&order, &payment, &taker);
        let min_maker_amount = self.calculate_min_maker_amount(
            &order.min_total_output,
            &order.initial_input_amount,
            &tokens_to_buy,
        );

        let result = self.process_p2p_fill(ProcessP2pFillArgs {
            maker: &order.maker,
            input_token: order.input_token.clone(),
            min_maker_amount,
            taker: &taker,
            tokens_to_buy: tokens_to_buy.clone(),
            taker_payment: &payment,
        });
        require!(
            matches!(result, ProcessP2pFillResult::Success),
            SENT_TOO_FEW_TOKENS_ERR_MSG
        );

        self.update_order_and_fire_events(order_id, &mut order, tokens_to_buy);
    }

    #[view(getTokensNeededForP2pBuyInput)]
    fn get_tokens_needed_for_p2p_buy_input(
        &self,
        order_id: OrderId,
        tokens_to_buy: &BigUint,
    ) -> BigUint {
        let order = self.orders(order_id).get();
        let protocol_fee_percent = self.p2p_protocol_fee().get();
        let payment_amount_needed =
            tokens_to_buy * &order.min_total_output / &order.initial_input_amount;

        payment_amount_needed * MAX_PERCENT / (MAX_PERCENT - protocol_fee_percent)
    }

    // TODO: Use this function somewhere?
    #[view(getTokensNeededForP2pSellInput)]
    fn get_tokens_needed_for_p2p_sell_input(
        &self,
        order_id: OrderId,
        payment_amount: &BigUint,
    ) -> BigUint {
        let order = self.orders(order_id).get();
        let protocol_fee_percent = self.p2p_protocol_fee().get();
        let tokens_to_buy = payment_amount * &order.initial_input_amount / &order.min_total_output;

        tokens_to_buy * MAX_PERCENT / (MAX_PERCENT - protocol_fee_percent)
    }

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

    fn get_tokens_to_buy_and_refund_surplus(
        &self,
        order: &Order<Self::Api>,
        payment: &EsdtTokenPayment,
        taker: &ManagedAddress,
    ) -> BigUint {
        let protocol_fee_percent = self.p2p_protocol_fee().get();
        let total_fees = &payment.amount * protocol_fee_percent / MAX_PERCENT;
        let remaining_payment_amount = &payment.amount - &total_fees;

        let tokens_to_buy =
            &remaining_payment_amount * &order.initial_input_amount / &order.min_total_output;
        if tokens_to_buy <= order.current_input_amount {
            return tokens_to_buy;
        }

        // TODO: Check if this is even correct???
        let surplus_output = &order.current_input_amount - &tokens_to_buy;
        let surplus_input = surplus_output * &order.initial_input_amount / &order.min_total_output;
        self.send().direct_non_zero_esdt_payment(
            taker,
            &EsdtTokenPayment::new(payment.token_identifier.clone(), 0, surplus_input),
        );

        order.current_input_amount.clone()
    }

    #[must_use]
    fn process_p2p_fill(&self, args: ProcessP2pFillArgs<Self::Api>) -> ProcessP2pFillResult {
        let protocol_fee_percent = self.p2p_protocol_fee().get();
        let total_protocol_fee = &args.taker_payment.amount * protocol_fee_percent / MAX_PERCENT;
        let remaining_tokens_maker = &args.taker_payment.amount - &total_protocol_fee;
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
            &args.maker,
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
