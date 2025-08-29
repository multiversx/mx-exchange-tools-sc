use crate::storage::order::OrderId;

multiversx_sc::imports!();

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

        let order_mapper = self.orders(order_id);
        let mut order = order_mapper.get();
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

        self.process_p2p_fill();

        // TODO: Update order
        // TODO: Event
    }

    /*
        - calculates the P2P protocol fee based on the Token A volume (sent to the treasury),
        - determines the net Token A for the Taker,
        - distributes Token B to the Maker,
        - and directs any surplus Token B (from Taker overpayment in specific scenarios) to the treasury.
        - It also updates the order's remaining amounts and status.
    */
    fn process_p2p_fill(&self) {}
}
