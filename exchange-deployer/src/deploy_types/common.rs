multiversx_sc::imports!();

pub const GAS_AFTER_ASYNC: u64 = 10_000;

#[multiversx_sc::module]
pub trait CommonModule {
    fn get_default_code_metadata(&self) -> CodeMetadata {
        CodeMetadata::PAYABLE_BY_SC | CodeMetadata::READABLE | CodeMetadata::UPGRADEABLE
    }

    fn require_deployed_contract(
        &self,
        user_address: &ManagedAddress,
        sc_address: &ManagedAddress,
    ) {
        require!(
            self.deployed_contracts(user_address).contains(sc_address),
            "Cannot perform action, contract deployed by another account"
        );
    }

    #[storage_mapper("deployedContracts")]
    fn deployed_contracts(
        &self,
        user_address: &ManagedAddress,
    ) -> UnorderedSetMapper<ManagedAddress>;
}
