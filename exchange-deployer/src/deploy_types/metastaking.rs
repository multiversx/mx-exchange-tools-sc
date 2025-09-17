use crate::action_type::DeployActionType;

multiversx_sc::imports!();

#[multiversx_sc::module]
pub trait MetastakingModule:
    crate::fee::FeeModule
    + super::common::CommonModule
    + utils::UtilsModule
    + multiversx_sc_modules::pause::PauseModule
{
    #[only_owner]
    #[endpoint(setMetastakingSourceAddress)]
    fn set_metastaking_source_address(&self, metastaking_source: ManagedAddress) {
        self.require_sc_address(&metastaking_source);

        self.metastaking_source().set(metastaking_source);
    }

    #[only_owner]
    #[endpoint(setEnergyFactoryAddress)]
    fn set_energy_factory_address(&self, energy_factory_address: ManagedAddress) {
        self.require_sc_address(&energy_factory_address);

        self.energy_factory_address().set(energy_factory_address);
    }

    #[payable("*")]
    #[endpoint(deployMetastaking)]
    fn deploy_metastaking(
        &self,
        lp_farm_address: ManagedAddress,
        staking_farm_address: ManagedAddress,
        pair_address: ManagedAddress,
        staking_token_id: TokenIdentifier,
        lp_farm_token_id: TokenIdentifier,
        staking_farm_token_id: TokenIdentifier,
        lp_token_id: TokenIdentifier,
    ) {
        self.require_not_paused();

        let caller = self.blockchain().get_caller();
        let payment = self.call_value().single_esdt();
        self.take_fee(&caller, payment, DeployActionType::Metastaking);

        let metastaking_source = self.metastaking_source().get();
        let energy_factory_address = self.energy_factory_address().get();
        let code_metadata = self.get_default_code_metadata();
        let (deployed_sc_address, ()) = self
            .metastaking_proxy()
            .init(
                energy_factory_address,
                lp_farm_address,
                staking_farm_address,
                pair_address,
                staking_token_id,
                lp_farm_token_id,
                staking_farm_token_id,
                lp_token_id,
            )
            .deploy_from_source(&metastaking_source, code_metadata);

        let _ = self.deployed_contracts(&caller).insert(deployed_sc_address);
    }

    #[proxy]
    fn metastaking_proxy(&self) -> farm_staking_proxy::Proxy<Self::Api>;

    #[storage_mapper("energyFactoryAddress")]
    fn energy_factory_address(&self) -> SingleValueMapper<ManagedAddress>;

    #[storage_mapper("metastakingSource")]
    fn metastaking_source(&self) -> SingleValueMapper<ManagedAddress>;
}
