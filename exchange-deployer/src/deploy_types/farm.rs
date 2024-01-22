use crate::action_type::DeployActionType;

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait FarmModule:
    crate::fee::FeeModule
    + super::common::CommonModule
    + utils::UtilsModule
    + multiversx_sc_modules::pause::PauseModule
{
    #[only_owner]
    #[endpoint(setFarmSourceAddress)]
    fn set_farm_source_address(&self, farm_source: ManagedAddress) {
        self.require_sc_address(&farm_source);

        self.farm_source().set(farm_source);
    }

    #[payable("*")]
    #[endpoint(deployFarm)]
    fn deploy_farm(
        &self,
        reward_token_id: TokenIdentifier,
        farming_token_id: TokenIdentifier,
        division_safety_constant: BigUint,
        pair_contract_address: ManagedAddress,
    ) {
        self.require_not_paused();

        let caller = self.blockchain().get_caller();
        let payment = self.call_value().single_esdt();
        self.take_fee(&caller, payment, DeployActionType::Farm);

        let own_sc_address = self.blockchain().get_sc_address();
        let mut admins = MultiValueEncoded::new();
        admins.push(caller.clone());

        let farm_source = self.farm_source().get();
        let code_metadata = self.get_default_code_metadata();
        let (deployed_sc_address, ()) = self
            .farm_proxy()
            .init(
                reward_token_id,
                farming_token_id,
                division_safety_constant,
                pair_contract_address,
                own_sc_address,
                admins,
            )
            .deploy_from_source(&farm_source, code_metadata);

        let _ = self.deployed_contracts(&caller).insert(deployed_sc_address);
    }

    #[proxy]
    fn farm_proxy(&self) -> farm::Proxy<Self::Api>;

    #[storage_mapper("farmSource")]
    fn farm_source(&self) -> SingleValueMapper<ManagedAddress>;
}
