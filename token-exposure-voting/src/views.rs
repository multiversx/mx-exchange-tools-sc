multiversx_sc::imports!();
multiversx_sc::derive_imports!();

use week_timekeeping::Week;

use crate::config::{BoostedToken, TokenRanking};

// Constants for shares-based boost calculations
pub const DIVISION_SAFETY_CONSTANT: u64 = 1_000_000_000_000;
pub const BOOST_COEF: u64 = DIVISION_SAFETY_CONSTANT / 2;

#[derive(TypeAbi, TopEncode)]
pub struct TokenRankPosition {
    pub token_position: usize,
    pub total_tokens: usize,
}

#[multiversx_sc::module]
pub trait ViewsModule: crate::config::ConfigModule {
    #[view(getBoostedTokensForWeek)]
    fn get_boosted_tokens_for_week(&self, week: Week) -> ManagedVec<BoostedToken<Self::Api>> {
        let mut boosted_tokens = ManagedVec::new();

        for token in self.tokens_for_week(week).iter() {
            let boost_amount = self.boosted_amount(&token, week).get();
            if boost_amount > 0 {
                boosted_tokens.push(BoostedToken {
                    token_id: token,
                    boost_amount,
                });
            }
        }

        boosted_tokens
    }

    #[view(getTokenRanking)]
    fn get_token_ranking(&self, token_id: TokenIdentifier, week: Week) -> TokenRankPosition {
        let ranking = self.get_week_ranking(week);
        let total_tokens = ranking.len();

        for (index, token_ranking) in ranking.iter().enumerate() {
            if token_ranking.token_id == token_id {
                return TokenRankPosition {
                    token_position: index + 1,
                    total_tokens,
                };
            }
        }

        // Token not found in ranking
        TokenRankPosition {
            token_position: 0,
            total_tokens,
        }
    }

    #[view(getWeekRanking)]
    fn get_week_ranking(&self, week: Week) -> ManagedVec<TokenRanking<Self::Api>> {
        let mut ranking = ManagedVec::new();

        for token in self.tokens_for_week(week).iter() {
            let votes = self.token_votes(&token, week).get();
            ranking.push(TokenRanking {
                token_id: token,
                votes,
            });
        }

        self.apply_boost_multipliers(&mut ranking, week);

        let len = ranking.len();
        for i in 0..len {
            let max_idx = self.find_max_votes_index(&ranking, i);
            self.swap_token_rankings(&mut ranking, i, max_idx);
        }

        ranking
    }

    fn apply_boost_multipliers(
        &self,
        ranking: &mut ManagedVec<TokenRanking<Self::Api>>,
        week: Week,
    ) {
        let total_boosted_amount = self.total_boosted_amount(week).get();

        if total_boosted_amount == 0 {
            return;
        }

        self.apply_shares_based_boost(ranking, week, &total_boosted_amount);
    }

    fn apply_shares_based_boost(
        &self,
        ranking: &mut ManagedVec<TokenRanking<Self::Api>>,
        week: Week,
        total_boosted_amount: &BigUint,
    ) {
        let div_safety = BigUint::from(DIVISION_SAFETY_CONSTANT);
        let boost_coef = BigUint::from(BOOST_COEF);

        for i in 0..ranking.len() {
            let mut token_ranking = ranking.get(i);
            let boosted_amount = self.boosted_amount(&token_ranking.token_id, week).get();

            if boosted_amount > 0 {
                // Calculate token_share scaled by div_safety: (boosted_amount / total_boosted_amount) * div_safety
                let token_share = &div_safety * &boosted_amount / total_boosted_amount;

                // Apply boost coefficient: token_share * boost_coef / div_safety
                let boosted_share = &token_share * &boost_coef / &div_safety;

                // Final multiplier: div_safety + boosted_share (both scaled by div_safety)
                let token_coef = &div_safety + &boosted_share;

                token_ranking.votes = &token_ranking.votes * &token_coef / &div_safety;
                let _ = ranking.set(i, &token_ranking);
            }
        }
    }

    fn find_max_votes_index(
        &self,
        ranking: &ManagedVec<TokenRanking<Self::Api>>,
        start_idx: usize,
    ) -> usize {
        let mut max_idx = start_idx;
        for j in start_idx + 1..ranking.len() {
            let current_votes = &ranking.get(j).votes;
            let max_votes = &ranking.get(max_idx).votes;
            if current_votes > max_votes {
                max_idx = j;
            }
        }
        max_idx
    }

    fn swap_token_rankings(
        &self,
        ranking: &mut ManagedVec<TokenRanking<Self::Api>>,
        i: usize,
        max_idx: usize,
    ) {
        if max_idx != i {
            let temp = ranking.get(i);
            let max_item = ranking.get(max_idx);
            let _ = ranking.set(i, &max_item);
            let _ = ranking.set(max_idx, &temp);
        }
    }
}
