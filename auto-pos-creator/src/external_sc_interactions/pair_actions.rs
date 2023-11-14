multiversx_sc::imports!();

use pair::{AddLiquidityResultType, RemoveLiquidityResultType};

pub const SWAP_MIN_AMOUNT: u64 = 1;

pub struct PairAddLiqResult<M: ManagedTypeApi> {
    pub lp_tokens: EsdtTokenPayment<M>,
    pub first_tokens_remaining: EsdtTokenPayment<M>,
    pub second_tokens_remaining: EsdtTokenPayment<M>,
}

pub struct PairRemoveLiqResult<M: ManagedTypeApi> {
    pub first_tokens: EsdtTokenPayment<M>,
    pub second_tokens: EsdtTokenPayment<M>,
}

pub type PairTokenPayments<M> = PairRemoveLiqResult<M>;

#[multiversx_sc::module]
pub trait PairActionsModule:
    crate::configs::pairs_config::PairsConfigModule + utils::UtilsModule
{
    fn call_pair_swap(
        &self,
        pair_address: ManagedAddress,
        input_tokens: EsdtTokenPayment,
        requested_token_id: TokenIdentifier,
    ) -> EsdtTokenPayment {
        self.pair_proxy(pair_address)
            .swap_tokens_fixed_input(requested_token_id, BigUint::from(SWAP_MIN_AMOUNT))
            .with_esdt_transfer(input_tokens)
            .execute_on_dest_context()
    }

    fn call_pair_add_liquidity(
        &self,
        pair_address: ManagedAddress,
        first_tokens: EsdtTokenPayment,
        second_tokens: EsdtTokenPayment,
        first_token_min_amount_out: BigUint,
        second_token_min_amount_out: BigUint,
    ) -> PairAddLiqResult<Self::Api> {
        let raw_results: AddLiquidityResultType<Self::Api> = self
            .pair_proxy(pair_address)
            .add_liquidity(first_token_min_amount_out, second_token_min_amount_out)
            .with_esdt_transfer(first_tokens)
            .with_esdt_transfer(second_tokens)
            .execute_on_dest_context();

        let (lp_tokens, first_tokens_remaining, second_tokens_remaining) = raw_results.into_tuple();

        PairAddLiqResult {
            lp_tokens,
            first_tokens_remaining,
            second_tokens_remaining,
        }
    }

    fn call_pair_remove_liquidity(
        &self,
        pair_address: ManagedAddress,
        lp_tokens: EsdtTokenPayment,
        first_token_min_amount_out: BigUint,
        second_token_min_amount_out: BigUint,
    ) -> PairRemoveLiqResult<Self::Api> {
        let raw_results: RemoveLiquidityResultType<Self::Api> = self
            .pair_proxy(pair_address)
            .remove_liquidity(first_token_min_amount_out, second_token_min_amount_out)
            .with_esdt_transfer(lp_tokens)
            .execute_on_dest_context();
        let (first_tokens, second_tokens) = raw_results.into_tuple();

        PairRemoveLiqResult {
            first_tokens,
            second_tokens,
        }
    }

    #[proxy]
    fn pair_proxy(&self, sc_address: ManagedAddress) -> pair::Proxy<Self::Api>;
}
