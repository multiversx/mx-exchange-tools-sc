use crate::user_data::action::action_types::{
    ActionId, ActionInfo, NrRetries, TotalActions, TradeFrequency,
};

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait EditActionModule:
    crate::user_data::ids::IdsModule
    + crate::user_data::funds::FundsModule
    + super::storage::ActionStorageModule
    + crate::events::EventsModule
    + utils::UtilsModule
    + multiversx_sc_modules::pause::PauseModule
{
    #[only_owner]
    #[endpoint(setNrRetries)]
    fn set_nr_retries(&self, nr_retries: NrRetries) {
        self.nr_retries().set(nr_retries);
    }

    #[endpoint(addTotalActions)]
    fn add_total_actions(&self, action_id: ActionId, to_add: TotalActions) {
        self.require_not_paused();

        let caller_id = self.get_caller_id_strict();
        self.action_info(action_id).update(|action_info| {
            self.require_correct_caller_id(action_info, caller_id);

            action_info.total_actions_left += to_add;
        });

        self.emit_add_total_actions_event(action_id, to_add);
    }

    #[endpoint(removeTotalActions)]
    fn remove_total_actions(&self, action_id: ActionId, to_remove: TotalActions) {
        self.require_not_paused();

        let caller_id = self.get_caller_id_strict();
        let action_mapper = self.action_info(action_id);
        let (actions_left, action_in_progress) = action_mapper.update(|action_info| {
            self.require_correct_caller_id(action_info, caller_id);

            if action_info.total_actions_left > to_remove {
                action_info.total_actions_left -= to_remove;
            } else {
                action_info.total_actions_left = 0;
            }

            if action_info.action_in_progress {
                action_info.total_actions_left += 1;
            }

            (
                action_info.total_actions_left,
                action_info.action_in_progress,
            )
        });

        if actions_left == 0 && !action_in_progress {
            action_mapper.clear();
        }

        self.emit_remove_total_actions_event(action_id, to_remove);
    }

    #[endpoint(changeTradeFrequency)]
    fn change_trade_frequency(&self, action_id: ActionId, new_trade_freq: TradeFrequency) {
        self.require_not_paused();

        let caller_id = self.get_caller_id_strict();
        let old_trade_freq = self.action_info(action_id).update(|action_info| {
            self.require_correct_caller_id(action_info, caller_id);
            require!(
                action_info.trade_frequency != new_trade_freq,
                "Same trade frequency as before"
            );

            let old_trade_freq = action_info.trade_frequency;
            action_info.trade_frequency = new_trade_freq;

            old_trade_freq
        });

        self.emit_change_trade_freq_event(action_id, old_trade_freq, new_trade_freq);
    }

    fn get_caller_id_strict(&self) -> AddressId {
        let caller = self.blockchain().get_caller();
        self.user_ids().get_id_non_zero(&caller)
    }

    fn require_correct_caller_id(&self, action_info: &ActionInfo<Self::Api>, caller_id: AddressId) {
        require!(
            action_info.owner_id == caller_id,
            "Invalid action ID or don't own the action"
        );
    }
}
