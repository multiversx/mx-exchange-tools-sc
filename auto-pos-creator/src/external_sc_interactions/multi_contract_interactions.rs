elrond_wasm::imports!();

#[elrond_wasm::module]
pub trait MultiContractInteractionsModule:
    super::pair_actions::PairActionsModule
    + crate::configs::pairs_config::PairsConfigModule
    + utils::UtilsModule
{
    #[endpoint(createPosFromSingleToken)]
    fn create_pos_from_single_token(&self, dest_pair_address: ManagedAddress) {
        let payment = self.call_value().single_esdt();
        let double_swap_result = self.buy_half_each_token(payment, &dest_pair_address);
        let _add_liq_result = self.call_pair_add_liquidity(
            dest_pair_address,
            double_swap_result.first_swap_tokens,
            double_swap_result.second_swap_tokens,
        );
    }
}
