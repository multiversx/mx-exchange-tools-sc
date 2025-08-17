multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait VoteModule:
    crate::config::ConfigModule
    + week_timekeeping::WeekTimekeepingModule
    + energy_query::EnergyQueryModule
{
    #[endpoint(vote)]
    fn vote(&self, token_id: TokenIdentifier) {
        let caller = self.blockchain().get_caller();
        let current_week = self.get_current_week();

        require!(
            !self.user_has_voted(current_week).contains(&caller),
            "User has already voted this week"
        );
        let user_energy = self.get_energy_amount_non_zero(&caller);

        let _ = self.tokens_for_week(current_week).insert(token_id.clone());
        self.token_votes(&token_id, current_week).update(|votes| {
            *votes += &user_energy;
        });
        self.total_votes(current_week).update(|total| {
            *total += &user_energy;
        });
        self.user_has_voted(current_week).add(&caller);
    }

    #[payable("*")]
    #[endpoint(boost)]
    fn boost(&self, token_id: TokenIdentifier) {
        let payment = self.call_value().single_esdt();
        let expected_token = self.voting_token_id().get();

        require!(
            payment.token_identifier == expected_token,
            "Wrong token for boosting"
        );

        let current_week = self.get_current_week();

        self.boosted_amount(&token_id, current_week)
            .update(|amount| *amount += &payment.amount);

        self.total_boosted_amount(current_week)
            .update(|total| *total += &payment.amount);
    }

    #[only_owner]
    #[endpoint(withdrawBoostFunds)]
    fn withdraw_boost_funds(&self) {
        let caller = self.blockchain().get_caller();
        let voting_token = self.voting_token_id().get();
        let balance = self
            .blockchain()
            .get_sc_balance(&EgldOrEsdtTokenIdentifier::esdt(voting_token.clone()), 0);

        require!(balance > 0, "No boost funds to withdraw");

        self.send().direct_esdt(&caller, &voting_token, 0, &balance);
    }
}
