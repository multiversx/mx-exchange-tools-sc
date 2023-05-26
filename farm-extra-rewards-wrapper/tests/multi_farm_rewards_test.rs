use farm_extra_rewards_setup::ExtraRewSetup;
use farm_extra_rewards_wrapper::{
    reward_tokens::RewardTokensModule,
    wrapped_farm_attributes::WrappedFarmAttributes,
    wrapper_actions::{
        generate_rewards::GenerateRewardsModule, wrap_farm_token::WrapFarmTokenModule,
    },
};
use multiversx_sc::{codec::Empty, types::EsdtLocalRole};
use multiversx_sc_scenario::{
    managed_address, managed_biguint, managed_token_id, rust_biguint, DebugApi,
};
use pausable::{PausableModule, State};
use tests_common::farm_with_locked_rewards_setup::{
    FarmSetup, FARMING_TOKEN_BALANCE, FARM_TOKEN_ID,
};

use config::ConfigModule;
use farm_token::FarmTokenModule;
use multiversx_sc::storage::mappers::StorageTokenWrapper;
use sc_whitelist_module::SCWhitelistModule;

pub mod farm_extra_rewards_setup;

pub static WRAPPED_FARM_TOKEN_ID: &[u8] = b"WRAPPED-123456";
pub static FIRST_REW_TOKEN: &[u8] = b"FIRSTREW-123456";
pub static SECOND_REW_TOKEN: &[u8] = b"SECONDREW-123456";
pub const REW_TOKEN_BALANCE: u64 = 1_000_000_000_000_000_000;
pub const PER_BLOCK_REWARD_AMOUNT: u64 = 1_000_000_000;

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

    farm_setup.b_mock.borrow_mut().set_esdt_balance(
        &extra_rew_setup.owner,
        FIRST_REW_TOKEN,
        &rust_biguint!(REW_TOKEN_BALANCE),
    );
    farm_setup.b_mock.borrow_mut().set_esdt_balance(
        &extra_rew_setup.owner,
        SECOND_REW_TOKEN,
        &rust_biguint!(REW_TOKEN_BALANCE),
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

#[test]
fn deposit_rewards_test() {
    let _ = DebugApi::dummy();
    let (farm_setup, extra_rew_setup) = setup_all(
        farm_with_locked_rewards::contract_obj,
        energy_factory::contract_obj,
        farm_extra_rewards_wrapper::contract_obj,
    );

    let b_mock = farm_setup.b_mock;
    b_mock.borrow_mut().set_block_nonce(10);

    b_mock
        .borrow_mut()
        .execute_esdt_transfer(
            &extra_rew_setup.owner,
            &extra_rew_setup.sc_wrapper,
            FIRST_REW_TOKEN,
            0,
            &rust_biguint!(REW_TOKEN_BALANCE),
            |sc| {
                let _ = sc.deposit_reward_tokens();
            },
        )
        .assert_ok();

    b_mock
        .borrow_mut()
        .execute_esdt_transfer(
            &extra_rew_setup.owner,
            &extra_rew_setup.sc_wrapper,
            SECOND_REW_TOKEN,
            0,
            &rust_biguint!(REW_TOKEN_BALANCE),
            |sc| {
                let _ = sc.deposit_reward_tokens();
            },
        )
        .assert_ok();
}

#[test]
fn claim_rewards_test() {
    let _ = DebugApi::dummy();
    let (mut farm_setup, extra_rew_setup) = setup_all(
        farm_with_locked_rewards::contract_obj,
        energy_factory::contract_obj,
        farm_extra_rewards_wrapper::contract_obj,
    );
    farm_setup.enter_farm(0, &farm_setup.first_user.clone(), FARMING_TOKEN_BALANCE);
    farm_setup.enter_farm(1, &farm_setup.first_user.clone(), FARMING_TOKEN_BALANCE);

    let b_mock = farm_setup.b_mock;
    b_mock.borrow_mut().set_block_nonce(10);

    // deposit first token rewards
    b_mock
        .borrow_mut()
        .execute_esdt_transfer(
            &extra_rew_setup.owner,
            &extra_rew_setup.sc_wrapper,
            FIRST_REW_TOKEN,
            0,
            &rust_biguint!(REW_TOKEN_BALANCE),
            |sc| {
                let _ = sc.deposit_reward_tokens();
            },
        )
        .assert_ok();

    // deposit second token rewards
    b_mock
        .borrow_mut()
        .execute_esdt_transfer(
            &extra_rew_setup.owner,
            &extra_rew_setup.sc_wrapper,
            SECOND_REW_TOKEN,
            0,
            &rust_biguint!(REW_TOKEN_BALANCE),
            |sc| {
                let _ = sc.deposit_reward_tokens();
            },
        )
        .assert_ok();

    // set rewards info
    b_mock
        .borrow_mut()
        .execute_tx(
            &extra_rew_setup.owner,
            &extra_rew_setup.sc_wrapper,
            &rust_biguint!(0),
            |sc| {
                sc.per_block_reward_amount()
                    .set(&managed_biguint!(PER_BLOCK_REWARD_AMOUNT));

                sc.state().set(State::Active);
                sc.produce_rewards_enabled().set(true);
            },
        )
        .assert_ok();

    // wrap first farm token
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

    // wrap second farm token
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

    b_mock.borrow_mut().set_block_nonce(20);

    // claim rewards
    b_mock
        .borrow_mut()
        .execute_esdt_transfer(
            &farm_setup.first_user,
            &extra_rew_setup.sc_wrapper,
            WRAPPED_FARM_TOKEN_ID,
            1,
            &rust_biguint!(FARMING_TOKEN_BALANCE),
            |sc| {
                let _ = sc.claim_rewards();
            },
        )
        .assert_ok();

    b_mock
        .borrow_mut()
        .execute_esdt_transfer(
            &farm_setup.first_user,
            &extra_rew_setup.sc_wrapper,
            WRAPPED_FARM_TOKEN_ID,
            2,
            &rust_biguint!(FARMING_TOKEN_BALANCE),
            |sc| {
                let _ = sc.claim_rewards();
            },
        )
        .assert_ok();

    // check user NFTs
    b_mock.borrow().check_nft_balance::<Empty>(
        &farm_setup.first_user,
        WRAPPED_FARM_TOKEN_ID,
        1,
        &rust_biguint!(0),
        None,
    );
    b_mock.borrow().check_nft_balance::<Empty>(
        &farm_setup.first_user,
        WRAPPED_FARM_TOKEN_ID,
        2,
        &rust_biguint!(0),
        None,
    );
    b_mock.borrow().check_nft_balance(
        &farm_setup.first_user,
        WRAPPED_FARM_TOKEN_ID,
        3,
        &rust_biguint!(FARMING_TOKEN_BALANCE),
        Some(&WrappedFarmAttributes::<DebugApi> {
            farm_token_id: managed_token_id!(FARM_TOKEN_ID[0]),
            farm_token_nonce: 2,
            creation_block: 20,
            current_token_amount: managed_biguint!(FARMING_TOKEN_BALANCE),
            reward_per_share: managed_biguint!(0x09184e72a000),
        }),
    );
    b_mock.borrow().check_nft_balance(
        &farm_setup.first_user,
        WRAPPED_FARM_TOKEN_ID,
        4,
        &rust_biguint!(FARMING_TOKEN_BALANCE),
        Some(&WrappedFarmAttributes::<DebugApi> {
            farm_token_id: managed_token_id!(FARM_TOKEN_ID[1]),
            farm_token_nonce: 2,
            creation_block: 20,
            current_token_amount: managed_biguint!(FARMING_TOKEN_BALANCE),
            reward_per_share: managed_biguint!(0x09184e72a000),
        }),
    );

    // check user reward balance
    b_mock.borrow().check_esdt_balance(
        &farm_setup.first_user,
        FIRST_REW_TOKEN,
        &rust_biguint!(20_000_000_000), // 20 blocks * 1_000_000_000
    );
    b_mock.borrow().check_esdt_balance(
        &farm_setup.first_user,
        SECOND_REW_TOKEN,
        &rust_biguint!(20_000_000_000), // 20 blocks * 1_000_000_000
    );
}

#[test]
fn claim_rewards_half_test() {
    let _ = DebugApi::dummy();
    let (mut farm_setup, extra_rew_setup) = setup_all(
        farm_with_locked_rewards::contract_obj,
        energy_factory::contract_obj,
        farm_extra_rewards_wrapper::contract_obj,
    );
    farm_setup.enter_farm(0, &farm_setup.first_user.clone(), FARMING_TOKEN_BALANCE);
    farm_setup.enter_farm(1, &farm_setup.first_user.clone(), FARMING_TOKEN_BALANCE);

    let b_mock = farm_setup.b_mock;
    b_mock.borrow_mut().set_block_nonce(10);

    // deposit first token rewards
    b_mock
        .borrow_mut()
        .execute_esdt_transfer(
            &extra_rew_setup.owner,
            &extra_rew_setup.sc_wrapper,
            FIRST_REW_TOKEN,
            0,
            &rust_biguint!(REW_TOKEN_BALANCE),
            |sc| {
                let _ = sc.deposit_reward_tokens();
            },
        )
        .assert_ok();

    // deposit second token rewards
    b_mock
        .borrow_mut()
        .execute_esdt_transfer(
            &extra_rew_setup.owner,
            &extra_rew_setup.sc_wrapper,
            SECOND_REW_TOKEN,
            0,
            &rust_biguint!(REW_TOKEN_BALANCE),
            |sc| {
                let _ = sc.deposit_reward_tokens();
            },
        )
        .assert_ok();

    // set rewards info
    b_mock
        .borrow_mut()
        .execute_tx(
            &extra_rew_setup.owner,
            &extra_rew_setup.sc_wrapper,
            &rust_biguint!(0),
            |sc| {
                sc.per_block_reward_amount()
                    .set(&managed_biguint!(PER_BLOCK_REWARD_AMOUNT));

                sc.state().set(State::Active);
                sc.produce_rewards_enabled().set(true);
            },
        )
        .assert_ok();

    // wrap first farm token
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

    // wrap second farm token
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

    b_mock.borrow_mut().set_block_nonce(20);

    // claim rewards
    b_mock
        .borrow_mut()
        .execute_esdt_transfer(
            &farm_setup.first_user,
            &extra_rew_setup.sc_wrapper,
            WRAPPED_FARM_TOKEN_ID,
            2,
            &rust_biguint!(FARMING_TOKEN_BALANCE / 2),
            |sc| {
                let _ = sc.claim_rewards();
            },
        )
        .assert_ok();

    // check user NFTs
    b_mock.borrow().check_nft_balance::<Empty>(
        &farm_setup.first_user,
        WRAPPED_FARM_TOKEN_ID,
        2,
        &rust_biguint!(FARMING_TOKEN_BALANCE / 2),
        None,
    );
    b_mock.borrow().check_nft_balance(
        &farm_setup.first_user,
        WRAPPED_FARM_TOKEN_ID,
        3,
        &rust_biguint!(FARMING_TOKEN_BALANCE / 2),
        Some(&WrappedFarmAttributes::<DebugApi> {
            farm_token_id: managed_token_id!(FARM_TOKEN_ID[1]),
            farm_token_nonce: 2,
            creation_block: 20,
            current_token_amount: managed_biguint!(FARMING_TOKEN_BALANCE / 2),
            reward_per_share: managed_biguint!(0x09184e72a000),
        }),
    );

    // check user reward balance
    b_mock.borrow().check_esdt_balance(
        &farm_setup.first_user,
        FIRST_REW_TOKEN,
        &rust_biguint!(5_000_000_000), // 20 blocks * 1_000_000_000 / 4
    );
    b_mock.borrow().check_esdt_balance(
        &farm_setup.first_user,
        SECOND_REW_TOKEN,
        &rust_biguint!(5_000_000_000), // 20 blocks * 1_000_000_000 / 4
    );
}
