multiversx_sc::imports!();

use week_timekeeping::Week;

use crate::config::{BoostedToken, TokenRanking};

// Constants for boost multiplier calculations
const PRECISION: u64 = 1_000_000u64;
const BOOST_FACTOR_NUMERATOR: u64 = 500_000u64; // (50% factor)
const BASE_MULTIPLIER: u64 = 1_500_000u64; // (150% base multiplier)

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
    fn get_token_ranking(
        &self,
        token_id: TokenIdentifier,
        week: Week,
    ) -> MultiValue2<usize, usize> {
        let ranking = self.get_week_ranking(week);
        let total_tokens = ranking.len();

        for (index, token_ranking) in ranking.iter().enumerate() {
            if token_ranking.token_id == token_id {
                return (index + 1, total_tokens).into();
            }
        }

        // Token not found in ranking
        (0, total_tokens).into()
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
            let mut max_idx = i;
            for j in i + 1..len {
                let current_votes = &ranking.get(j).votes;
                let max_votes = &ranking.get(max_idx).votes;
                if current_votes > max_votes {
                    max_idx = j;
                }
            }
            if max_idx != i {
                // Swap elements
                let temp = ranking.get(i).clone();
                let max_item = ranking.get(max_idx).clone();
                let _ = ranking.set(i, &max_item);
                let _ = ranking.set(max_idx, &temp);
            }
        }

        ranking
    }

    fn apply_boost_multipliers(
        &self,
        ranking: &mut ManagedVec<TokenRanking<Self::Api>>,
        week: Week,
    ) {
        let mut boosted_tokens = ManagedVec::<Self::Api, BoostedToken<Self::Api>>::new();

        for token_ranking in ranking.iter() {
            let boost_amount = self.boosted_amount(&token_ranking.token_id, week).get();
            if boost_amount > 0 {
                boosted_tokens.push(BoostedToken {
                    token_id: token_ranking.token_id,
                    boost_amount,
                });
            }
        }

        if boosted_tokens.is_empty() {
            return;
        }

        // Sort boosted tokens by their original vote count (descending)
        let boosted_len = boosted_tokens.len();
        for i in 0..boosted_len {
            let mut max_idx = i;
            for j in i + 1..boosted_len {
                let current_votes = self
                    .token_votes(&boosted_tokens.get(j).token_id, week)
                    .get();
                let max_votes = self
                    .token_votes(&boosted_tokens.get(max_idx).token_id, week)
                    .get();
                if current_votes > max_votes {
                    max_idx = j;
                }
            }
            if max_idx != i {
                let temp = boosted_tokens.get(i);
                let max_item = boosted_tokens.get(max_idx);
                let _ = boosted_tokens.set(i, &max_item);
                let _ = boosted_tokens.set(max_idx, &temp);
            }
        }

        let num_boosted = BigUint::from(boosted_tokens.len());
        let boost_factor_numerator = BigUint::from(BOOST_FACTOR_NUMERATOR);
        let precision = BigUint::from(PRECISION);
        let boost_factor = boost_factor_numerator / &num_boosted; // in thousandths

        // Apply boost multipliers
        for (index, boosted_token) in boosted_tokens.iter().enumerate() {
            let base_multiplier = BigUint::from(BASE_MULTIPLIER);
            let reduction = &boost_factor * &BigUint::from(index);
            let final_multiplier = base_multiplier - reduction;

            // Find and update the corresponding token in ranking
            for i in 0..ranking.len() {
                let mut token_ranking = ranking.get(i);
                if token_ranking.token_id == boosted_token.token_id {
                    // Apply multiplier: votes = votes * final_multiplier / 1000
                    token_ranking.votes = &token_ranking.votes * &final_multiplier / &precision;
                    let _ = ranking.set(i, &token_ranking);
                    break;
                }
            }
        }
    }
}
