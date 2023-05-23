use farm_extra_rewards_setup::ExtraRewSetup;
use farm_extra_rewards_wrapper::{
    wrapped_farm_attributes::WrappedFarmAttributes,
    wrapper_actions::wrap_farm_token::WrapFarmTokenModule,
};
use multiversx_sc::{codec::Empty, types::EsdtLocalRole};
use multiversx_sc_scenario::{
    managed_address, managed_biguint, managed_token_id, rust_biguint, DebugApi,
};
use pausable::PausableModule;
use tests_common::farm_with_locked_rewards_setup::{
    FarmSetup, FARMING_TOKEN_BALANCE, FARM_TOKEN_ID,
};

use farm_token::FarmTokenModule;
use multiversx_sc::storage::mappers::StorageTokenWrapper;
use sc_whitelist_module::SCWhitelistModule;

pub mod farm_extra_rewards_setup;

pub static WRAPPED_FARM_TOKEN_ID: &[u8] = b"WRAPPED-123456";

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

    farm_setup
        .b_mock
        .borrow_mut()
        .execute_tx(
            &extra_rew_setup.owner,
            &extra_rew_setup.sc_wrapper,
            &rust_biguint!(0),
            |sc| {
                sc.resume();
                sc.farm_token()
                    .set_token_id(managed_token_id!(WRAPPED_FARM_TOKEN_ID));
            },
        )
        .assert_ok();

    farm_setup.b_mock.borrow_mut().set_esdt_local_roles(
        &extra_rew_setup.sc_wrapper.address_ref(),
        WRAPPED_FARM_TOKEN_ID,
        &[EsdtLocalRole::NftCreate, EsdtLocalRole::NftBurn],
    );

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

#[test]
fn enter_farm_test() {
    let _ = DebugApi::dummy();
    let (mut farm_setup, extra_rew_setup) = setup_all(
        farm_with_locked_rewards::contract_obj,
        energy_factory::contract_obj,
        farm_extra_rewards_wrapper::contract_obj,
    );
    farm_setup.enter_farm(0, &farm_setup.first_user.clone(), FARMING_TOKEN_BALANCE);
    farm_setup.enter_farm(1, &farm_setup.first_user.clone(), FARMING_TOKEN_BALANCE);

    let b_mock = farm_setup.b_mock;
    b_mock.borrow().check_nft_balance::<Empty>(
        &farm_setup.first_user,
        FARM_TOKEN_ID[0],
        1,
        &rust_biguint!(FARMING_TOKEN_BALANCE),
        None,
    );
    b_mock.borrow().check_nft_balance::<Empty>(
        &farm_setup.first_user,
        FARM_TOKEN_ID[1],
        1,
        &rust_biguint!(FARMING_TOKEN_BALANCE),
        None,
    );

    b_mock.borrow_mut().set_block_nonce(10);

    b_mock
        .borrow_mut()
        .execute_esdt_transfer(
            &farm_setup.first_user,
            &extra_rew_setup.sc_wrapper,
            FARM_TOKEN_ID[0],
            1,
            &rust_biguint!(FARMING_TOKEN_BALANCE),
            |sc| {
                let _ = sc.wrap_farm_token_endpoint();
            },
        )
        .assert_ok();

    b_mock.borrow_mut().set_block_nonce(20);

    b_mock
        .borrow_mut()
        .execute_esdt_transfer(
            &farm_setup.first_user,
            &extra_rew_setup.sc_wrapper,
            FARM_TOKEN_ID[1],
            1,
            &rust_biguint!(FARMING_TOKEN_BALANCE),
            |sc| {
                let _ = sc.wrap_farm_token_endpoint();
            },
        )
        .assert_ok();

    b_mock.borrow().check_nft_balance(
        &farm_setup.first_user,
        WRAPPED_FARM_TOKEN_ID,
        1,
        &rust_biguint!(FARMING_TOKEN_BALANCE),
        Some(&WrappedFarmAttributes::<DebugApi> {
            farm_token_id: managed_token_id!(FARM_TOKEN_ID[0]),
            farm_token_nonce: 1,
            creation_block: 10,
            current_token_amount: managed_biguint!(FARMING_TOKEN_BALANCE),
            reward_per_share: managed_biguint!(0),
        }),
    );

    b_mock.borrow().check_nft_balance(
        &farm_setup.first_user,
        WRAPPED_FARM_TOKEN_ID,
        2,
        &rust_biguint!(FARMING_TOKEN_BALANCE),
        Some(&WrappedFarmAttributes::<DebugApi> {
            farm_token_id: managed_token_id!(FARM_TOKEN_ID[1]),
            farm_token_nonce: 1,
            creation_block: 20,
            current_token_amount: managed_biguint!(FARMING_TOKEN_BALANCE),
            reward_per_share: managed_biguint!(0),
        }),
    );
}
