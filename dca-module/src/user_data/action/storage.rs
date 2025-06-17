use crate::user_data::action::action_types::{ActionId, ActionInfo, NrRetries};

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait ActionStorageModule {
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
