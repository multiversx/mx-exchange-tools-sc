multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[derive(TypeAbi, TopEncode)]
pub struct SmartSwapEvent<M: ManagedTypeApi> {
    caller: ManagedAddress<M>,
    token_in: TokenIdentifier<M>,
    amount_in: BigUint<M>,
    token_out: TokenIdentifier<M>,
    amount_out: BigUint<M>,
    fee_amount: BigUint<M>,
    block: u64,
    epoch: u64,
    timestamp: u64,
}

#[multiversx_sc::module]
pub trait EventsModule {
    fn emit_smart_swap_event(
        &self,
        caller: ManagedAddress,
        token_in: EsdtTokenPayment,
        token_out: EsdtTokenPayment,
        fee_amount: BigUint,
    ) {
        let block = self.blockchain().get_block_nonce();
        let epoch = self.blockchain().get_block_epoch();
        let timestamp = self.blockchain().get_block_timestamp();

        self.smart_swap_event(
            caller.clone(),
            token_in.token_identifier.clone(),
            token_in.amount.clone(),
            token_out.token_identifier.clone(),
            token_in.amount.clone(),
            epoch,
            SmartSwapEvent {
                caller,
                token_in: token_in.token_identifier,
                amount_in: token_in.amount,
                token_out: token_out.token_identifier,
                amount_out: token_out.amount,
                fee_amount,
                block,
                epoch,
                timestamp,
            },
        )
    }

    #[event("SmartSwap")]
    fn smart_swap_event(
        &self,
        #[indexed] caller: ManagedAddress,
        #[indexed] token_in: TokenIdentifier,
        #[indexed] amount_in: BigUint,
        #[indexed] token_out: TokenIdentifier,
        #[indexed] amount_out: BigUint,
        #[indexed] epoch: u64,
        smart_swap_event: SmartSwapEvent<Self::Api>,
    );
}
