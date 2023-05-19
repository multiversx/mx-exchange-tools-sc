use farm_extra_rewards_setup::ExtraRewSetup;
use multiversx_sc_scenario::{managed_address, rust_biguint, DebugApi};
use tests_common::farm_with_locked_rewards_setup::FarmSetup;

use sc_whitelist_module::SCWhitelistModule;

pub mod farm_extra_rewards_setup;

fn setup_all<FarmObjBuilder, EnergyFactoryBuilder, ExtraRewardsBuilder>(
    farm_builder: FarmObjBuilder,
    energy_factory_builder: EnergyFactoryBuilder,
    extra_rewards_builder: ExtraRewardsBuilder,
) -> (
    FarmSetup<FarmObjBuilder, EnergyFactoryBuilder>,
    ExtraRewSetup<ExtraRewardsBuilder>,
)
where
    FarmObjBuilder: 'static + Copy + Fn() -> farm_with_locked_rewards::ContractObj<DebugApi>,
    EnergyFactoryBuilder: 'static + Copy + Fn() -> energy_factory::ContractObj<DebugApi>,
    ExtraRewardsBuilder: 'static + Copy + Fn() -> farm_extra_rewards_wrapper::ContractObj<DebugApi>,
{
    let farm_setup = FarmSetup::new(farm_builder, energy_factory_builder);
    let mut extra_rew_setup = ExtraRewSetup::new(
        farm_setup.b_mock.clone(),
        farm_setup.owner.clone(),
        extra_rewards_builder,
    );
    extra_rew_setup.add_farms(vec![
        farm_setup.farm_wrappers[0].address_ref().clone(),
        farm_setup.farm_wrappers[1].address_ref().clone(),
    ]);

    for wrapper in &farm_setup.farm_wrappers {
        farm_setup
            .b_mock
            .borrow_mut()
            .execute_tx(&farm_setup.owner, wrapper, &rust_biguint!(0), |sc| {
                sc.add_sc_address_to_whitelist(managed_address!(extra_rew_setup
                    .sc_wrapper
                    .address_ref()));
            })
            .assert_ok();
    }

    (farm_setup, extra_rew_setup)
}

#[test]
fn init_test() {
    let _ = setup_all(
        farm_with_locked_rewards::contract_obj,
        energy_factory::contract_obj,
        farm_extra_rewards_wrapper::contract_obj,
    );
}
