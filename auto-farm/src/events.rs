use common_structs::PaymentsVec;

use crate::common::{chain_info::CurrentChainInfo, rewards_wrapper::MergedRewardsWrapper};

elrond_wasm::imports!();
elrond_wasm::derive_imports!();

#[derive(TypeAbi, TopEncode)]
pub struct TokenDepositEvent<'a, M: ManagedTypeApi> {
    pub tokens: &'a PaymentsVec<M>,
    pub chain_info: CurrentChainInfo,
}

#[derive(TypeAbi, TopEncode)]
pub enum DepositType {
    FarmTokens,
    MetastakingTokens,
}

#[derive(TypeAbi, TopEncode)]
pub struct TokenWithdrawalEvent<'a, M: ManagedTypeApi> {
    pub tokens: &'a PaymentsVec<M>,
    pub chain_info: CurrentChainInfo,
}

#[derive(TypeAbi, TopEncode)]
pub enum WithdrawType {
    FarmTokens,
    MetastakingTokens,
    RewardTokens,
    AllTokens,
}

#[derive(TypeAbi, TopEncode)]
pub struct ProxyClaimEvent<'a, M: ManagedTypeApi> {
    pub available_rewards: &'a MergedRewardsWrapper<M>,
    pub updated_user_tokens: &'a PaymentsVec<M>,
    pub chain_info: CurrentChainInfo,
}

#[elrond_wasm::module]
pub trait EventsModule {
    fn emit_user_register_event(&self, user: &ManagedAddress) {
        self.user_register_event(user, CurrentChainInfo::new::<Self::Api>())
    }

    fn emit_token_deposit_event(
        &self,
        user: &ManagedAddress,
        deposit_type: DepositType,
        tokens: &PaymentsVec<Self::Api>,
    ) {
        self.token_deposit_event(
            user,
            deposit_type,
            TokenDepositEvent {
                tokens,
                chain_info: CurrentChainInfo::new::<Self::Api>(),
            },
        );
    }

    fn emit_token_withdrawal_event(
        &self,
        user: &ManagedAddress,
        withdraw_type: WithdrawType,
        tokens: &PaymentsVec<Self::Api>,
    ) {
        self.token_withdrawal_event(
            user,
            withdraw_type,
            TokenWithdrawalEvent {
                tokens,
                chain_info: CurrentChainInfo::new::<Self::Api>(),
            },
        );
    }

    fn emit_proxy_claim_event(
        &self,
        user: &ManagedAddress,
        available_rewards: &MergedRewardsWrapper<Self::Api>,
        updated_user_tokens: &PaymentsVec<Self::Api>,
    ) {
        self.proxy_claim_event(
            user,
            ProxyClaimEvent {
                available_rewards,
                updated_user_tokens,
                chain_info: CurrentChainInfo::new::<Self::Api>(),
            },
        );
    }

    #[event("userRegister")]
    fn user_register_event(
        &self,
        #[indexed] user: &ManagedAddress,
        current_chain_info: CurrentChainInfo,
    );

    #[event("tokenDeposit")]
    fn token_deposit_event(
        &self,
        #[indexed] user: &ManagedAddress,
        #[indexed] deposit_type: DepositType,
        event_data: TokenDepositEvent<Self::Api>,
    );

    #[event("tokenWithdrawal")]
    fn token_withdrawal_event(
        &self,
        #[indexed] user: &ManagedAddress,
        #[indexed] withdraw_type: WithdrawType,
        event_data: TokenWithdrawalEvent<Self::Api>,
    );

    #[event("proxyClaim")]
    fn proxy_claim_event(
        &self,
        #[indexed] user: &ManagedAddress,
        event_data: ProxyClaimEvent<Self::Api>,
    );
}
