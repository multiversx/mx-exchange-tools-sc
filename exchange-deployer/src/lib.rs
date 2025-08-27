#![no_std]

multiversx_sc::imports!();

pub mod action_type;
pub mod deploy_types;
pub mod fee;

#[multiversx_sc::contract]
pub trait ExchangeDeployer:
    fee::FeeModule
    + deploy_types::liquidity_pool::LiquidityPoolModule
    + deploy_types::simple_lock::SimpleLockModule
    + deploy_types::proxy_dex::ProxyDexModule
    + deploy_types::farm::FarmModule
    + deploy_types::farm_staking::FarmStakingModule
    + deploy_types::metastaking::MetastakingModule
    + deploy_types::common::CommonModule
    + utils::UtilsModule
    + multiversx_sc_modules::pause::PauseModule
{
    #[init]
    fn init(
        &self,
        default_action_fee: BigUint,
        pair_source_address: ManagedAddress,
        simple_lock_source_address: ManagedAddress,
        proxy_dex_source_address: ManagedAddress,
        farm_source_address: ManagedAddress,
        farm_staking_source_address: ManagedAddress,
        metastaking_source_address: ManagedAddress,
    ) {
        self.set_paused(true);

        self.set_default_action_fee(default_action_fee);
        self.set_pair_source_address(pair_source_address);
        self.set_simple_lock_source_address(simple_lock_source_address);
        self.set_proxy_dex_source_address(proxy_dex_source_address);
        self.set_farm_source_address(farm_source_address);
        self.set_farm_staking_source_address(farm_staking_source_address);
        self.set_metastaking_source_address(metastaking_source_address);
    }

    #[upgrade]
    fn upgrade(&self) {}
}
