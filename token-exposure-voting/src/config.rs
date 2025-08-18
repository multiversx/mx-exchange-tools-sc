use week_timekeeping::Week;

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[derive(ManagedVecItem, TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi, Clone)]
pub struct TokenRanking<M: ManagedTypeApi> {
    pub token_id: TokenIdentifier<M>,
    pub votes: BigUint<M>,
}

#[derive(ManagedVecItem, TopEncode, TopDecode, NestedEncode, NestedDecode, TypeAbi, Clone)]
pub struct BoostedToken<M: ManagedTypeApi> {
    pub token_id: TokenIdentifier<M>,
    pub boost_amount: BigUint<M>,
}

#[multiversx_sc::module]
pub trait ConfigModule {
    #[storage_mapper("userHasVoted")]
    fn user_has_voted(&self, week: Week) -> WhitelistMapper<ManagedAddress>;

    #[view(getTokenVotes)]
    #[storage_mapper("tokenVotes")]
    fn token_votes(&self, token_id: &TokenIdentifier, week: Week) -> SingleValueMapper<BigUint>;

    #[view(getTotalVotes)]
    #[storage_mapper("totalVotes")]
    fn total_votes(&self, week: Week) -> SingleValueMapper<BigUint>;

    #[view(getTokensForWeek)]
    #[storage_mapper("tokensForWeek")]
    fn tokens_for_week(&self, week: Week) -> UnorderedSetMapper<TokenIdentifier>;

    #[view(getVotingTokenId)]
    #[storage_mapper("votingTokenId")]
    fn voting_token_id(&self) -> SingleValueMapper<TokenIdentifier>;

    #[view(getBoostedAmount)]
    #[storage_mapper("boostedAmount")]
    fn boosted_amount(&self, token_id: &TokenIdentifier, week: Week) -> SingleValueMapper<BigUint>;

    #[view(getTotalBoostedAmount)]
    #[storage_mapper("totalBoostedAmount")]
    fn total_boosted_amount(&self, week: Week) -> SingleValueMapper<BigUint>;
}
