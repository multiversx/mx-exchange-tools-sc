use crate::router_actions::SwapOperationType;

multiversx_sc::imports!();
multiversx_sc::derive_imports!();

pub type ActionId = u64;
pub type NrRetries = usize;

#[derive(TypeAbi, TopEncode, TopDecode)]
pub struct Action<M: ManagedTypeApi> {
    pub pair_address: ManagedAddress<M>,
    pub endpoint_name: ManagedBuffer<M>,
    pub requested_token: TokenIdentifier<M>,
    pub min_amount_out: BigUint<M>,
}

impl<M: ManagedTypeApi> From<SwapOperationType<M>> for Action<M> {
    fn from(value: SwapOperationType<M>) -> Self {
        let (pair_address, endpoint_name, requested_token, min_amount_out) = value.into_tuple();

        Self {
            pair_address,
            endpoint_name,
            requested_token,
            min_amount_out,
        }
    }
}

impl<M: ManagedTypeApi> From<Action<M>> for SwapOperationType<M> {
    #[inline]
    fn from(value: Action<M>) -> Self {
        (
            value.pair_address,
            value.endpoint_name,
            value.requested_token,
            value.min_amount_out,
        )
            .into()
    }
}

#[multiversx_sc::module]
pub trait ActionModule: super::ids::IdsModule {
    #[only_owner]
    #[endpoint(setNrRetries)]
    fn set_nr_retries(&self, nr_retries: NrRetries) {
        require!(nr_retries > 0, "Invalid nr retries");

        self.nr_retries().set(nr_retries);
    }

    fn increment_and_get_action_id(&self) -> ActionId {
        self.action_id().update(|action_id| {
            *action_id += 1;

            *action_id
        })
    }

    // action ID "0" unused
    #[storage_mapper("actionId")]
    fn action_id(&self) -> SingleValueMapper<ActionId>;

    #[storage_mapper("nrRetries")]
    fn nr_retries(&self) -> SingleValueMapper<NrRetries>;

    // TODO: Clear this value after successful execution
    #[storage_mapper("nrRetriesPerAction")]
    fn nr_retries_per_action(&self, action_id: ActionId) -> SingleValueMapper<NrRetries>;
}
