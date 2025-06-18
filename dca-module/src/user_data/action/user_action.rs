use super::action_types::{
    router_arg_from_user_arg, Action, ActionId, ActionInfo, SwapOperationTypeUserArg, TotalActions,
    TradeFrequency,
};

multiversx_sc::imports!();

// TODO: Adapt if needed
pub const MAX_SWAP_ACTIONS: TotalActions = 5;

#[multiversx_sc::module]
pub trait ActionModule:
    crate::user_data::ids::IdsModule
    + crate::user_data::funds::FundsModule
    + super::storage::ActionStorageModule
    + crate::router_actions::RouterActionsModule
    + utils::UtilsModule
    + multiversx_sc_modules::pause::PauseModule
{
    #[endpoint(registerAction)]
    fn register_action(
        &self,
        trade_frequency: TradeFrequency,
        total_actions: TotalActions,
        input_tokens_id: TokenIdentifier,
        input_tokens_amount: BigUint,
        swap_args: MultiValueEncoded<SwapOperationTypeUserArg<Self::Api>>,
    ) {
        require!(total_actions > 0, "May not specifiy action with 0 tries");
        require!(
            input_tokens_id.is_valid_esdt_identifier(),
            "Invalid token ID"
        );
        require!(input_tokens_amount > 0, "Invalid token amount");
        require!(!swap_args.is_empty(), "No swap args provided");
        require!(swap_args.len() <= MAX_SWAP_ACTIONS, "Too many swap actions");

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
            input_tokens_id,
            input_tokens_amount,
            last_action_timestamp: current_time,
            total_actions_left: total_actions,
            action_in_progress: false,
            actions: actions_vec,
        };

        self.action_info(action_id).set(action_info);

        // TODO: event
    }

    #[endpoint(executeAction)]
    fn execute_action(&self, action_id: ActionId) {
        let action_mapper = self.action_info(action_id);
        require!(
            !action_mapper.is_empty(),
            "Action either already executed or doesn't exist"
        );

        let mut action_info = action_mapper.get();
        require!(
            !action_info.action_in_progress,
            "Action currently in progress. Awaiting callback..."
        );

        let current_timestamp = self.blockchain().get_block_timestamp();
        let next_action_timestamp = action_info.get_next_action_timestamp();
        require!(
            current_timestamp >= next_action_timestamp,
            "Trying to execute action too early"
        );

        action_info.action_in_progress = true;

        let funds_mapper = self.user_funds(action_info.owner_id);
        require!(!funds_mapper.is_empty(), "No funds deposited by user");

        let payment_for_action = EsdtTokenPayment::new(
            action_info.input_tokens_id.clone(),
            0,
            action_info.input_tokens_amount.clone(),
        );
        funds_mapper.update(|user_funds| {
            let deduct_result = user_funds.deduct_payment(&payment_for_action);
            require!(deduct_result.is_ok(), "User does not have enough funds");
        });

        action_mapper.set(&action_info);
        self.nr_retries_per_action(action_id)
            .update(|nr_retries| *nr_retries += 1);

        let mut swap_operations = MultiValueEncoded::new();
        for action in &action_info.actions {
            let swap_arg = action.into();
            swap_operations.push(swap_arg);
        }

        let user_address = unsafe {
            self.user_ids()
                .get_address(action_info.owner_id)
                .unwrap_unchecked()
        };
        self.call_router_swap(action_id, user_address, payment_for_action, swap_operations);
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

    fn get_or_insert_caller_id(&self) -> AddressId {
        let caller = self.blockchain().get_caller();
        self.user_ids().get_id_or_insert(&caller)
    }

    fn increment_and_get_action_id(&self) -> ActionId {
        self.last_action_id().update(|action_id| {
            *action_id += 1;

            *action_id
        })
    }
}
