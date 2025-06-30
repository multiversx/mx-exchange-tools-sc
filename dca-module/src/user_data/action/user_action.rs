use super::action_types::{
    router_arg_from_user_arg, ActionId, ActionInfo, SwapOperationTypeUserArg, TotalActions,
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
        input_token_id: TokenIdentifier,
        input_tokens_amount: BigUint,
        output_token_id: TokenIdentifier,
    ) {
        require!(total_actions > 0, "May not specifiy action with 0 tries");
        require!(
            input_token_id.is_valid_esdt_identifier(),
            "Invalid input token ID"
        );
        require!(input_tokens_amount > 0, "Invalid token amount");
        require!(
            output_token_id.is_valid_esdt_identifier(),
            "Invalid output token ID"
        );

        let caller_id = self.get_or_insert_caller_id();
        let action_id = self.increment_and_get_action_id();

        let current_time = self.blockchain().get_block_timestamp();
        let action_info = ActionInfo {
            owner_id: caller_id,
            trade_frequency,
            input_token_id,
            input_tokens_amount,
            output_token_id,
            last_action_timestamp: current_time,
            total_actions_left: total_actions,
            action_in_progress: false,
        };

        self.action_info(action_id).set(action_info);

        // TODO: event
    }

    #[endpoint(executeAction)]
    fn execute_action(
        &self,
        action_id: ActionId,
        swap_args: MultiValueEncoded<SwapOperationTypeUserArg<Self::Api>>,
    ) {
        require!(!swap_args.is_empty(), "No swap args provided");
        require!(swap_args.len() <= MAX_SWAP_ACTIONS, "Too many swap actions");

        let mut action_info = self.try_get_action_info(action_id);
        self.require_correct_first_token_id(swap_args.clone(), &action_info.input_token_id);
        self.require_correct_last_token_id(swap_args.clone(), &action_info.output_token_id);

        let current_timestamp = self.blockchain().get_block_timestamp();
        let next_action_timestamp = action_info.get_next_action_timestamp();
        require!(
            current_timestamp >= next_action_timestamp,
            "Trying to execute action too early"
        );

        action_info.action_in_progress = true;

        self.action_info(action_id).set(&action_info);
        self.nr_retries_per_action(action_id)
            .update(|nr_retries| *nr_retries += 1);

        let mut swap_operations = MultiValueEncoded::new();
        for swap_arg in swap_args {
            let router_swap_arg = router_arg_from_user_arg(swap_arg);
            swap_operations.push(router_swap_arg);
        }

        let user_address = unsafe {
            self.user_ids()
                .get_address(action_info.owner_id)
                .unwrap_unchecked()
        };
        let payment_for_action = self.subtract_and_get_payment_for_action(&action_info);
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

    fn try_get_action_info(&self, action_id: ActionId) -> ActionInfo<Self::Api> {
        let action_mapper = self.action_info(action_id);
        require!(
            !action_mapper.is_empty(),
            "Action either already executed or doesn't exist"
        );

        let action_info = action_mapper.get();
        require!(
            !action_info.action_in_progress,
            "Action currently in progress. Awaiting callback..."
        );

        action_info
    }

    fn require_correct_first_token_id(
        &self,
        swap_args: MultiValueEncoded<SwapOperationTypeUserArg<Self::Api>>,
        required_first_token_id: &TokenIdentifier,
    ) {
        let (_, first_token_id, _) =
            unsafe { swap_args.into_iter().next().unwrap_unchecked().into_tuple() };
        require!(
            &first_token_id == required_first_token_id,
            "Invalid first token"
        );
    }

    fn require_correct_last_token_id(
        &self,
        swap_args: MultiValueEncoded<SwapOperationTypeUserArg<Self::Api>>,
        output_token_id: &TokenIdentifier,
    ) {
        let (_, last_token_id, _) =
            unsafe { swap_args.into_iter().last().unwrap_unchecked().into_tuple() };
        require!(&last_token_id == output_token_id, "Invalid output token");
    }

    fn subtract_and_get_payment_for_action(
        &self,
        action_info: &ActionInfo<Self::Api>,
    ) -> EsdtTokenPayment {
        let funds_mapper = self.user_funds(action_info.owner_id);
        require!(!funds_mapper.is_empty(), "No funds deposited by user");

        let payment_for_action = EsdtTokenPayment::new(
            action_info.input_token_id.clone(),
            0,
            action_info.input_tokens_amount.clone(),
        );
        funds_mapper.update(|user_funds| {
            let deduct_result = user_funds.deduct_payment(&payment_for_action);
            require!(deduct_result.is_ok(), "User does not have enough funds");
        });

        payment_for_action
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
