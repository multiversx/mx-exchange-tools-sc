use crate::storage::{
    common_storage::MAX_PERCENT,
    order::{Order, OrderId},
};

multiversx_sc::imports!();

pub struct ProcessP2pFillArgs<'a, M: ManagedTypeApi> {
    pub order: &'a mut Order<M>,
    pub min_maker_amount: &'a BigUint<M>,
    pub taker: &'a ManagedAddress<M>,
    pub tokens_to_buy: &'a BigUint<M>,
    pub taker_payment: &'a EsdtTokenPayment<M>,
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
    fn fill_order_p2p_by_buying_input(&self, order_id: OrderId, nr_tokens_to_buy: BigUint) {
        self.require_not_paused();
        self.require_valid_order_id(order_id);

        let mut order = self.orders(order_id).get();
        let payment = self.call_value().single_esdt();
        require!(
            payment.token_identifier == order.output_token,
            "Invalid token sent"
        );
        require!(
            nr_tokens_to_buy <= order.current_input_amount,
            "Buying too many tokens"
        );

        let min_maker_amount = self.calculate_min_maker_amount(
            &order.min_total_output,
            &order.initial_input_amount,
            &nr_tokens_to_buy,
        );
        require!(payment.amount >= min_maker_amount, "Sent too few tokens");

        let taker = self.blockchain().get_caller();
        self.process_p2p_fill(ProcessP2pFillArgs {
            order: &mut order,
            min_maker_amount: &min_maker_amount,
            taker: &taker,
            tokens_to_buy: &nr_tokens_to_buy,
            taker_payment: &payment,
        });

        self.update_order_and_fire_events(order_id, &mut order, nr_tokens_to_buy);
    }

    fn process_p2p_fill(&self, args: ProcessP2pFillArgs<Self::Api>) {
        let protocol_fee_percent = self.p2p_protocol_fee().get();
        let total_protocol_fee = args.tokens_to_buy * protocol_fee_percent / MAX_PERCENT;
        let remaining_tokens_taker = args.tokens_to_buy - &total_protocol_fee;

        let treasury_addresss = self.treasury_address().get();
        self.send().direct_non_zero_esdt_payment(
            &treasury_addresss,
            &EsdtTokenPayment::new(args.order.input_token.clone(), 0, total_protocol_fee),
        );

        let surplus = &args.taker_payment.amount - args.min_maker_amount;
        self.send().direct_non_zero_esdt_payment(
            &treasury_addresss,
            &EsdtTokenPayment::new(args.order.output_token.clone(), 0, surplus),
        );

        self.send()
            .direct_non_zero_esdt_payment(&args.order.maker, args.taker_payment);
        self.send().direct_non_zero_esdt_payment(
            args.taker,
            &EsdtTokenPayment::new(args.order.input_token.clone(), 0, remaining_tokens_taker),
        );
    }
}
