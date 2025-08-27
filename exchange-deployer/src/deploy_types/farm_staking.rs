use crate::action_type::DeployActionType;

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait FarmStakingModule:
    crate::fee::FeeModule
    + super::common::CommonModule
    + utils::UtilsModule
    + multiversx_sc_modules::pause::PauseModule
{
    #[only_owner]
    #[endpoint(setFarmStakingSourceAddress)]
    fn set_farm_staking_source_address(&self, farm_staking_source: ManagedAddress) {
        self.require_sc_address(&farm_staking_source);

        self.farm_staking_source().set(farm_staking_source);
    }

    #[payable("*")]
    #[endpoint(deployFarmStaking)]
    fn deploy_farm_staking(
        &self,
        farming_token_id: TokenIdentifier,
        division_safety_constant: BigUint,
        max_apr: BigUint,
        min_unbond_epochs: u64,
    ) {
        self.require_not_paused();

        let caller = self.blockchain().get_caller();
        let payment = self.call_value().single_esdt();
        self.take_fee(&caller, payment, DeployActionType::FarmStaking);

        let own_sc_address = self.blockchain().get_sc_address();
        let mut admins = MultiValueEncoded::new();
        admins.push(caller.clone());

        let farm_staking_source = self.farm_staking_source().get();
        let code_metadata = self.get_default_code_metadata();
        let (deployed_sc_address, ()) = self
            .farm_staking_proxy()
            .init(
                farming_token_id,
                division_safety_constant,
                max_apr,
                min_unbond_epochs,
                own_sc_address,
                admins,
            )
            .deploy_from_source(&farm_staking_source, code_metadata);

        let _ = self.deployed_contracts(&caller).insert(deployed_sc_address);
    }

    #[proxy]
    fn farm_staking_proxy(&self) -> farm_staking::Proxy<Self::Api>;

    #[storage_mapper("farmStakingSource")]
    fn farm_staking_source(&self) -> SingleValueMapper<ManagedAddress>;
}
