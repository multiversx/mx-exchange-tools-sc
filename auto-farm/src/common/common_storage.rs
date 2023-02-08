use super::address_to_id_mapper::AddressToIdMapper;

multiversx_sc::imports!();

pub const MAX_PERCENTAGE: u64 = 10_000;

#[multiversx_sc::module]
pub trait CommonStorageModule {
    fn require_caller_proxy_claim_address(&self) {
        let caller = self.blockchain().get_caller();
        let proxy_claim_address = self.proxy_claim_address().get();
        require!(
            caller == proxy_claim_address,
            "Only the proxy can claim in user's place"
        );
    }

    #[storage_mapper("userIds")]
    fn user_ids(&self) -> AddressToIdMapper<Self::Api>;

    #[storage_mapper("proxyClaimAddress")]
    fn proxy_claim_address(&self) -> SingleValueMapper<ManagedAddress>;
}
