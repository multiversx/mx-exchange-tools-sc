use crate::user_data::action_types::{
    router_arg_from_user_arg, Action, ActionId, ActionInfo, NrRetries, SwapOperationTypeUserArg,
    TotalActions, TradeFrequency,
};

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait ActionModule:
    super::ids::IdsModule
    + super::funds::FundsModule
    + utils::UtilsModule
    + multiversx_sc_modules::pause::PauseModule
{
    #[only_owner]
    #[endpoint(setNrRetries)]
    fn set_nr_retries(&self, nr_retries: NrRetries) {
        self.nr_retries().set(nr_retries);
    }

    #[endpoint(registerAction)]
    fn register_action(
        &self,
        trade_frequency: TradeFrequency,
        total_actions: TotalActions,
        swap_args: MultiValueEncoded<SwapOperationTypeUserArg<Self::Api>>,
    ) {
        require!(!swap_args.is_empty(), "No swap args provided");
        require!(total_actions > 0, "May not specifiy action with 0 tries");

        let caller_id = self.get_or_insert_caller_id();
        let action_id = self.increment_and_get_action_id();

        let mut actions_vec = ManagedVec::new();
        for swap_arg in swap_args {
            let router_swap_arg = router_arg_from_user_arg(swap_arg);
            let action = Action::from(router_swap_arg);

            actions_vec.push(action);
        }

        let current_time = self.blockchain().get_block_timestamp();
        let action_info = ActionInfo {
            owner_id: caller_id,
            trade_frequency,
            last_action_timestamp: current_time,
            total_actions_left: total_actions,
            action_in_progress: false,
            actions: actions_vec,
        };

        self.action_info(action_id).set(action_info);

        // TODO: event
    }

    #[endpoint(addTotalActions)]
    fn add_total_actions(&self, action_id: ActionId, to_add: TotalActions) {
        let caller_id = self.get_caller_id_strict();
        self.action_info(action_id).update(|action_info| {
            self.require_correct_caller_id(action_info, caller_id);

            action_info.total_actions_left += to_add;
        });

        // TODO: event
    }

    #[endpoint(removeTotalActions)]
    fn remove_total_actions(&self, action_id: ActionId, to_remove: TotalActions) {
        let caller_id = self.get_caller_id_strict();
        let action_mapper = self.action_info(action_id);
        let actions_left = action_mapper.update(|action_info| {
            self.require_correct_caller_id(action_info, caller_id);

            if action_info.total_actions_left > to_remove {
                action_info.total_actions_left -= to_remove;
            } else {
                action_info.total_actions_left = 0;
            }

            action_info.total_actions_left
        });

        if actions_left == 0 {
            action_mapper.clear();
        }

        // TODO: event
    }

    #[endpoint(changeTradeFrequency)]
    fn change_trade_frequency(&self, action_id: ActionId, new_trade_freq: TradeFrequency) {
        let caller_id = self.get_caller_id_strict();
        self.action_info(action_id).update(|action_info| {
            self.require_correct_caller_id(action_info, caller_id);
            require!(
                action_info.trade_frequency != new_trade_freq,
                "Same trade frequency as before"
            );

            action_info.trade_frequency = new_trade_freq;
        });

        // TODO: event
    }

    /// action ID "0" unused
    #[view(getActionInfo)]
    fn get_action_info(&self, action_id: ActionId) -> OptionalValue<ActionInfo<Self::Api>> {
        let mapper = self.action_info(action_id);
        if !mapper.is_empty() {
            OptionalValue::Some(mapper.get())
        } else {
            OptionalValue::None
        }
    }

    fn get_caller_id_strict(&self) -> AddressId {
        let caller = self.blockchain().get_caller();
        self.user_ids().get_id_non_zero(&caller)
    }

    fn get_or_insert_caller_id(&self) -> AddressId {
        let caller = self.blockchain().get_caller();
        self.user_ids().get_id_or_insert(&caller)
    }

    fn require_correct_caller_id(&self, action_info: &ActionInfo<Self::Api>, caller_id: AddressId) {
        require!(
            action_info.owner_id == caller_id,
            "Invalid action ID or don't own the action"
        );
    }

    fn increment_and_get_action_id(&self) -> ActionId {
        self.action_id().update(|action_id| {
            *action_id += 1;

            *action_id
        })
    }

    // action ID "0" unused
    #[view(getLastActionId)]
    #[storage_mapper("actionId")]
    fn action_id(&self) -> SingleValueMapper<ActionId>;

    // TODO: Clear when total_actions_left is 0
    #[storage_mapper("actionInfo")]
    fn action_info(&self, action_id: ActionId) -> SingleValueMapper<ActionInfo<Self::Api>>;

    #[view(getNrRetries)]
    #[storage_mapper("nrRetries")]
    fn nr_retries(&self) -> SingleValueMapper<NrRetries>;

    // TODO: Clear this value after successful execution
    #[storage_mapper("nrRetriesPerAction")]
    fn nr_retries_per_action(&self, action_id: ActionId) -> SingleValueMapper<NrRetries>;
}
