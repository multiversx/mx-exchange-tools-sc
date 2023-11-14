multiversx_sc::imports!();

pub struct PairConfig<M: ManagedTypeApi> {
    pub lp_token_id: TokenIdentifier<M>,
    pub first_token_id: TokenIdentifier<M>,
    pub second_token_id: TokenIdentifier<M>,
}

pub type SwapOperationType<M> =
    MultiValue4<ManagedAddress<M>, ManagedBuffer<M>, TokenIdentifier<M>, BigUint<M>>;

#[multiversx_sc::module]
pub trait PairsConfigModule: utils::UtilsModule {
    fn get_pair_config(&self, pair_address: &ManagedAddress) -> PairConfig<Self::Api> {
        let lp_token_id = self.lp_token_identifier().get_from_address(pair_address);
        let first_token_id = self.first_token_id().get_from_address(pair_address);
        let second_token_id = self.second_token_id().get_from_address(pair_address);

        self.require_valid_token_id(&lp_token_id);
        self.require_valid_token_id(&first_token_id);
        self.require_valid_token_id(&second_token_id);

        PairConfig {
            lp_token_id,
            first_token_id,
            second_token_id,
        }
    }

    // Pair storage

    #[storage_mapper("lpTokenIdentifier")]
    fn lp_token_identifier(&self) -> SingleValueMapper<TokenIdentifier>;

    #[storage_mapper("first_token_id")]
    fn first_token_id(&self) -> SingleValueMapper<TokenIdentifier>;

    #[storage_mapper("second_token_id")]
    fn second_token_id(&self) -> SingleValueMapper<TokenIdentifier>;
}
