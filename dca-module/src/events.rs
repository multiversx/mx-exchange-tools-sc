use auto_farm::common::chain_info::CurrentChainInfo;
use multiversx_sc_modules::transfer_role_proxy::PaymentsVec;

use crate::user_data::action::action_types::{
    ActionId, ActionInfo, NrRetries, Timestamp, TotalActions, TradeFrequency,
};

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[derive(TypeAbi, TopEncode)]
pub struct RegisterActionEvent<'a, M: ManagedTypeApi> {
    pub action_info: &'a ActionInfo<M>,
    pub chain_info: CurrentChainInfo,
}

#[derive(TypeAbi, TopEncode)]
pub struct DepositFundsEvent<'a, M: ManagedTypeApi> {
    pub funds: &'a PaymentsVec<M>,
    pub chain_info: CurrentChainInfo,
}

#[derive(TypeAbi, TopEncode)]
pub struct WithdrawFundsEvent<'a, M: ManagedTypeApi> {
    pub funds: &'a PaymentsVec<M>,
    pub chain_info: CurrentChainInfo,
}

#[derive(TypeAbi, TopEncode)]
pub struct AddTotalActionsEvent {
    pub actions_added: TotalActions,
    pub chain_info: CurrentChainInfo,
}

#[derive(TypeAbi, TopEncode)]
pub struct RemoveTotalActionsEvent {
    pub actions_removed: TotalActions,
    pub chain_info: CurrentChainInfo,
}

#[derive(TypeAbi, TopEncode)]
pub struct ChangeTradeFrequencyEvent {
    pub old_trade_freq: TradeFrequency,
    pub new_trade_freq: TradeFrequency,
    pub chain_info: CurrentChainInfo,
}

#[derive(TypeAbi, TopEncode)]
pub struct SwapSuccessEvent {
    pub execution_timestamp: Timestamp,
    pub actions_left: TotalActions,
    pub chain_info: CurrentChainInfo,
}

#[derive(TypeAbi, TopEncode)]
pub struct SwapFailedEvent {
    pub nr_retries: NrRetries,
    pub nr_retries_allowed: NrRetries,
    pub chain_info: CurrentChainInfo,
}

#[multiversx_sc::module]
pub trait EventsModule {
    fn emit_register_action_event(&self, action_id: ActionId, action_info: &ActionInfo<Self::Api>) {
        let caller = self.blockchain().get_caller();
        self.register_action_event(
            &caller,
            action_id,
            RegisterActionEvent {
                action_info,
                chain_info: CurrentChainInfo::new::<Self::Api>(),
            },
        );
    }

    fn emit_deposit_funds_event(&self, caller: &ManagedAddress, funds: &PaymentsVec<Self::Api>) {
        self.deposit_funds_event(
            caller,
            DepositFundsEvent {
                funds,
                chain_info: CurrentChainInfo::new::<Self::Api>(),
            },
        );
    }

    fn emit_withdraw_funds_event(&self, caller: &ManagedAddress, funds: &PaymentsVec<Self::Api>) {
        self.withdraw_funds_event(
            caller,
            WithdrawFundsEvent {
                funds,
                chain_info: CurrentChainInfo::new::<Self::Api>(),
            },
        );
    }

    fn emit_add_total_actions_event(&self, action_id: ActionId, actions_added: TotalActions) {
        let caller = self.blockchain().get_caller();
        self.add_total_actions_event(
            &caller,
            action_id,
            AddTotalActionsEvent {
                actions_added,
                chain_info: CurrentChainInfo::new::<Self::Api>(),
            },
        );
    }

    fn emit_remove_total_actions_event(&self, action_id: ActionId, actions_removed: TotalActions) {
        let caller = self.blockchain().get_caller();
        self.remove_total_actions_event(
            &caller,
            action_id,
            RemoveTotalActionsEvent {
                actions_removed,
                chain_info: CurrentChainInfo::new::<Self::Api>(),
            },
        );
    }

    fn emit_change_trade_freq_event(
        &self,
        action_id: ActionId,
        old_trade_freq: TradeFrequency,
        new_trade_freq: TradeFrequency,
    ) {
        let caller = self.blockchain().get_caller();
        self.change_trade_frequency_event(
            &caller,
            action_id,
            ChangeTradeFrequencyEvent {
                old_trade_freq,
                new_trade_freq,
                chain_info: CurrentChainInfo::new::<Self::Api>(),
            },
        );
    }

    fn emit_swap_success_event(
        &self,
        user: &ManagedAddress,
        action_id: ActionId,
        execution_timestamp: Timestamp,
        actions_left: TotalActions,
    ) {
        self.swap_success_event(
            user,
            action_id,
            SwapSuccessEvent {
                execution_timestamp,
                actions_left,
                chain_info: CurrentChainInfo::new::<Self::Api>(),
            },
        );
    }

    fn emit_swap_failed_event(
        &self,
        user: &ManagedAddress,
        action_id: ActionId,
        nr_retries: NrRetries,
        nr_retries_allowed: NrRetries,
    ) {
        self.swap_failed_event(
            user,
            action_id,
            SwapFailedEvent {
                nr_retries,
                nr_retries_allowed,
                chain_info: CurrentChainInfo::new::<Self::Api>(),
            },
        )
    }

    #[event("registerAction")]
    fn register_action_event(
        &self,
        #[indexed] user: &ManagedAddress,
        #[indexed] action_id: ActionId,
        event_data: RegisterActionEvent<Self::Api>,
    );

    #[event("depositFunds")]
    fn deposit_funds_event(
        &self,
        #[indexed] user: &ManagedAddress,
        event_data: DepositFundsEvent<Self::Api>,
    );

    #[event("withdrawFunds")]
    fn withdraw_funds_event(
        &self,
        #[indexed] user: &ManagedAddress,
        event_data: WithdrawFundsEvent<Self::Api>,
    );

    #[event("addTotalActions")]
    fn add_total_actions_event(
        &self,
        #[indexed] user: &ManagedAddress,
        #[indexed] action_id: ActionId,
        event_data: AddTotalActionsEvent,
    );

    #[event("removeTotalActions")]
    fn remove_total_actions_event(
        &self,
        #[indexed] user: &ManagedAddress,
        #[indexed] action_id: ActionId,
        event_data: RemoveTotalActionsEvent,
    );

    #[event("changeTradeFrequency")]
    fn change_trade_frequency_event(
        &self,
        #[indexed] user: &ManagedAddress,
        #[indexed] action_id: ActionId,
        event_data: ChangeTradeFrequencyEvent,
    );

    #[event("swapSuccessEvent")]
    fn swap_success_event(
        &self,
        #[indexed] user: &ManagedAddress,
        #[indexed] action_id: ActionId,
        event_data: SwapSuccessEvent,
    );

    #[event("swapFailedEvent")]
    fn swap_failed_event(
        &self,
        #[indexed] user: &ManagedAddress,
        #[indexed] action_id: ActionId,
        event_data: SwapFailedEvent,
    );
}
