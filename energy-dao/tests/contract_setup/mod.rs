#![allow(deprecated)]
#![allow(clippy::too_many_arguments)]
#![allow(dead_code)]

use farm::exit_penalty::ExitPenaltyModule;
use farm_staking::{custom_rewards::CustomRewardsModule, FarmStaking};
use farm_staking_proxy::{dual_yield_token::DualYieldTokenModule, FarmStakingProxy};
use farm_token::FarmTokenModule;
use farm_with_locked_rewards::Farm;
use fees_collector::FeesCollector;
use locked_token_wrapper::{wrapped_token::WrappedTokenModule, LockedTokenWrapper};
use multiversx_sc::types::{Address, EsdtLocalRole, ManagedAddress, MultiValueEncoded};
use multiversx_sc_modules::pause::PauseModule;
use multiversx_sc_scenario::{
    managed_address, managed_biguint, managed_token_id, rust_biguint, testing_framework::*,
    DebugApi,
};
use pair::{config::ConfigModule, Pair};

use config::ConfigModule as OtherConfigModule;
use energy_dao::{external_sc_interactions::energy_dao_config::EnergyDAOConfigModule, *};
use energy_factory::{locked_token_transfer::LockedTokenTransferModule, SimpleLockEnergy};
use energy_query::EnergyQueryModule;
use farm_boosted_yields::boosted_yields_factors::BoostedYieldsFactorsModule;
use locking_module::lock_with_energy_module::LockWithEnergyModule;
use pausable::{PausableModule, State};
use sc_whitelist_module::SCWhitelistModule;
use simple_lock::locked_token::LockedTokenModule;

pub const ENERGY_DAO_WASM_PATH: &str = "energy-dao/output/energy-dao.wasm";

// General
pub static WRAPPED_FARM_TOKEN_ID: &[u8] = b"WFARM-123456";
pub static UNSTAKE_TOKEN_ID: &[u8] = b"UNSTAKE-123456";
pub static WRAPPED_METASTAKING_TOKEN_ID: &[u8] = b"WMETA-123456";
pub static UNSTAKE_METASTAKING_TOKEN_ID: &[u8] = b"UMETA-123456";
pub static BASE_ASSET_TOKEN_ID: &[u8] = b"MEX-123456";
pub static FARMING_TOKEN_ID: &[u8] = b"LPTOK-123456";
pub static FARM_TOKEN_ID: &[u8] = b"FARM-123456";
pub static LOCKED_TOKEN_ID: &[u8] = b"LOCKED-123456";
pub static LEGACY_LOCKED_TOKEN_ID: &[u8] = b"LEGACY-123456";
pub static WRAPPED_LOCKED_TOKEN_ID: &[u8] = b"WLOCKED-123456";
pub static BASE_FARM_STAKING_TOKEN_ID: &[u8] = b"UTK-123456";
pub static OTHER_FARM_STAKING_TOKEN_ID: &[u8] = b"WEGLD-123456";
pub static STAKING_FARM_TOKEN_ID: &[u8] = b"SUTK-123456";
pub static DUAL_YIELD_TOKEN_ID: &[u8] = b"DUALYIELD-123456";
pub const PENALTY_PERCENTAGE: u64 = 300;
pub const MAX_APR_PERCENTAGE: u64 = 2_500;
pub const UNBOND_PERIOD: u64 = 10;
pub const EPOCHS_IN_YEAR: u64 = 360;
pub const USER_BALANCE: u64 = 1_000_000;
pub const DIVISION_SAFETY_CONSTANT: u64 = 1_000_000;

// Energy factory

pub static LOCK_OPTIONS: &[u64] = &[EPOCHS_IN_YEAR, 2 * EPOCHS_IN_YEAR, 4 * EPOCHS_IN_YEAR];
pub static PENALTY_PERCENTAGES: &[u64] = &[4_000, 6_000, 8_000];

// Farm
pub const PER_BLOCK_REWARD_AMOUNT: u64 = 5_000;
pub const TOTAL_REWARDS_AMOUNT: u64 = 1_000_000_000_000;
pub const BOOSTED_YIELDS_PERCENTAGE: u64 = 2_500; // 25%
pub const USER_REWARDS_BASE_CONST: u64 = 10;
pub const USER_REWARDS_ENERGY_CONST: u64 = 1;
pub const USER_REWARDS_FARM_CONST: u64 = 0;
pub const MIN_ENERGY_AMOUNT_FOR_BOOSTED_YIELDS: u64 = 1;
pub const MIN_FARM_AMOUNT_FOR_BOOSTED_YIELDS: u64 = 1;

pub static ESDT_ROLES: &[EsdtLocalRole] = &[EsdtLocalRole::Mint, EsdtLocalRole::Burn];

pub static SFT_ROLES: &[EsdtLocalRole] = &[
    EsdtLocalRole::NftCreate,
    EsdtLocalRole::NftAddQuantity,
    EsdtLocalRole::NftBurn,
];

pub static SFT_WITH_TRANSFER_ROLES: &[EsdtLocalRole] = &[
    EsdtLocalRole::NftCreate,
    EsdtLocalRole::NftAddQuantity,
    EsdtLocalRole::NftBurn,
    EsdtLocalRole::Transfer,
];

pub struct EnergyDAOContractSetup<
    EnergyDAOContractObjBuilder,
    EnergyFactoryObjBuilder,
    FeesCollectorObjBuilder,
    LockedTokenWrapperObjBuilder,
    PairObjBuilder,
    FarmObjBuilder,
    FarmStakingObjBuilder,
    FarmStakingProxyObjBuilder,
> where
    EnergyDAOContractObjBuilder: 'static + Copy + Fn() -> energy_dao::ContractObj<DebugApi>,
    EnergyFactoryObjBuilder: 'static + Copy + Fn() -> energy_factory::ContractObj<DebugApi>,
    FeesCollectorObjBuilder: 'static + Copy + Fn() -> fees_collector::ContractObj<DebugApi>,
    LockedTokenWrapperObjBuilder:
        'static + Copy + Fn() -> locked_token_wrapper::ContractObj<DebugApi>,
    PairObjBuilder: 'static + Copy + Fn() -> pair::ContractObj<DebugApi>,
    FarmObjBuilder: 'static + Copy + Fn() -> farm_with_locked_rewards::ContractObj<DebugApi>,
    FarmStakingObjBuilder: 'static + Copy + Fn() -> farm_staking::ContractObj<DebugApi>,
    FarmStakingProxyObjBuilder: 'static + Copy + Fn() -> farm_staking_proxy::ContractObj<DebugApi>,
{
    pub b_mock: BlockchainStateWrapper,
    pub owner_address: Address,
    pub energy_dao_wrapper:
        ContractObjWrapper<energy_dao::ContractObj<DebugApi>, EnergyDAOContractObjBuilder>,
    pub energy_factory_wrapper:
        ContractObjWrapper<energy_factory::ContractObj<DebugApi>, EnergyFactoryObjBuilder>,
    pub fees_collector_wrapper:
        ContractObjWrapper<fees_collector::ContractObj<DebugApi>, FeesCollectorObjBuilder>,
    pub locked_token_wrapper: ContractObjWrapper<
        locked_token_wrapper::ContractObj<DebugApi>,
        LockedTokenWrapperObjBuilder,
    >,
    pub pair_wrapper: ContractObjWrapper<pair::ContractObj<DebugApi>, PairObjBuilder>,
    pub farm_wrapper:
        ContractObjWrapper<farm_with_locked_rewards::ContractObj<DebugApi>, FarmObjBuilder>,
    pub farm_staking_wrapper:
        ContractObjWrapper<farm_staking::ContractObj<DebugApi>, FarmStakingObjBuilder>,
    pub farm_staking_proxy_wrapper:
        ContractObjWrapper<farm_staking_proxy::ContractObj<DebugApi>, FarmStakingProxyObjBuilder>,
}

impl<
        EnergyDAOContractObjBuilder,
        EnergyFactoryObjBuilder,
        FeesCollectorObjBuilder,
        LockedTokenWrapperObjBuilder,
        PairObjBuilder,
        FarmObjBuilder,
        FarmStakingObjBuilder,
        FarmStakingProxyObjBuilder,
    >
    EnergyDAOContractSetup<
        EnergyDAOContractObjBuilder,
        EnergyFactoryObjBuilder,
        FeesCollectorObjBuilder,
        LockedTokenWrapperObjBuilder,
        PairObjBuilder,
        FarmObjBuilder,
        FarmStakingObjBuilder,
        FarmStakingProxyObjBuilder,
    >
where
    EnergyDAOContractObjBuilder: 'static + Copy + Fn() -> energy_dao::ContractObj<DebugApi>,
    EnergyFactoryObjBuilder: 'static + Copy + Fn() -> energy_factory::ContractObj<DebugApi>,
    FeesCollectorObjBuilder: 'static + Copy + Fn() -> fees_collector::ContractObj<DebugApi>,
    LockedTokenWrapperObjBuilder:
        'static + Copy + Fn() -> locked_token_wrapper::ContractObj<DebugApi>,
    PairObjBuilder: 'static + Copy + Fn() -> pair::ContractObj<DebugApi>,
    FarmObjBuilder: 'static + Copy + Fn() -> farm_with_locked_rewards::ContractObj<DebugApi>,
    FarmStakingObjBuilder: 'static + Copy + Fn() -> farm_staking::ContractObj<DebugApi>,
    FarmStakingProxyObjBuilder: 'static + Copy + Fn() -> farm_staking_proxy::ContractObj<DebugApi>,
{
    pub fn new(
        energy_dao_builder: EnergyDAOContractObjBuilder,
        energy_factory_builder: EnergyFactoryObjBuilder,
        fees_collector_builder: FeesCollectorObjBuilder,
        locked_token_wrapper_builder: LockedTokenWrapperObjBuilder,
        pair_builder: PairObjBuilder,
        farm_builder: FarmObjBuilder,
        farm_staking_builder: FarmStakingObjBuilder,
        farm_staking_proxy_builder: FarmStakingProxyObjBuilder,
    ) -> Self {
        let rust_zero = rust_biguint!(0u64);
        let mut b_mock = BlockchainStateWrapper::new();
        let owner_address = b_mock.create_user_account(&rust_zero);

        let energy_dao_wrapper = b_mock.create_sc_account(
            &rust_zero,
            Some(&owner_address),
            energy_dao_builder,
            ENERGY_DAO_WASM_PATH,
        );

        let energy_factory_wrapper =
            setup_energy_factory(&mut b_mock, &owner_address, energy_factory_builder);

        let fees_collector_wrapper = setup_fees_collector(
            &mut b_mock,
            &owner_address,
            energy_factory_wrapper.address_ref(),
            fees_collector_builder,
        );

        let locked_token_wrapper = setup_locked_token_wrapper(
            &mut b_mock,
            &owner_address,
            &energy_factory_wrapper,
            locked_token_wrapper_builder,
        );

        let pair_wrapper = setup_pair(&mut b_mock, &owner_address, pair_builder);

        let farm_wrapper = setup_farm(
            &mut b_mock,
            &owner_address,
            pair_wrapper.address_ref(),
            &energy_factory_wrapper,
            farm_builder,
        );

        let farm_staking_wrapper =
            setup_farm_staking(&mut b_mock, &owner_address, farm_staking_builder);

        let farm_staking_proxy_wrapper = setup_farm_staking_proxy(
            &mut b_mock,
            &owner_address,
            energy_factory_wrapper.address_ref(),
            pair_wrapper.address_ref(),
            &farm_wrapper,
            &farm_staking_wrapper,
            farm_staking_proxy_builder,
        );

        b_mock
            .execute_tx(&owner_address, &energy_dao_wrapper, &rust_zero, |sc| {
                sc.init(
                    managed_address!(energy_factory_wrapper.address_ref()),
                    managed_address!(fees_collector_wrapper.address_ref()),
                    managed_address!(locked_token_wrapper.address_ref()),
                    PENALTY_PERCENTAGE,
                );

                sc.wrapped_farm_token()
                    .set_token_id(managed_token_id!(WRAPPED_FARM_TOKEN_ID));
                sc.unstake_farm_token()
                    .set_token_id(managed_token_id!(UNSTAKE_TOKEN_ID));
                sc.wrapped_metastaking_token()
                    .set_token_id(managed_token_id!(WRAPPED_METASTAKING_TOKEN_ID));
                sc.unstake_metastaking_token()
                    .set_token_id(managed_token_id!(UNSTAKE_METASTAKING_TOKEN_ID));
            })
            .assert_ok();

        b_mock.set_esdt_local_roles(
            energy_dao_wrapper.address_ref(),
            WRAPPED_FARM_TOKEN_ID,
            SFT_ROLES,
        );
        b_mock.set_esdt_local_roles(
            energy_dao_wrapper.address_ref(),
            UNSTAKE_TOKEN_ID,
            SFT_ROLES,
        );
        b_mock.set_esdt_local_roles(
            energy_dao_wrapper.address_ref(),
            WRAPPED_METASTAKING_TOKEN_ID,
            SFT_ROLES,
        );
        b_mock.set_esdt_local_roles(
            energy_dao_wrapper.address_ref(),
            UNSTAKE_METASTAKING_TOKEN_ID,
            SFT_ROLES,
        );

        b_mock.set_esdt_local_roles(energy_dao_wrapper.address_ref(), LOCKED_TOKEN_ID, SFT_ROLES);

        let wrapped_locked_reward_token_roles = [EsdtLocalRole::Transfer];
        b_mock.set_esdt_local_roles(
            energy_dao_wrapper.address_ref(),
            WRAPPED_LOCKED_TOKEN_ID,
            &wrapped_locked_reward_token_roles[..],
        );

        b_mock.set_esdt_balance(
            &owner_address,
            BASE_ASSET_TOKEN_ID,
            &rust_biguint!(USER_BALANCE),
        );

        EnergyDAOContractSetup {
            b_mock,
            owner_address,
            energy_dao_wrapper,
            energy_factory_wrapper,
            fees_collector_wrapper,
            locked_token_wrapper,
            pair_wrapper,
            farm_wrapper,
            farm_staking_wrapper,
            farm_staking_proxy_wrapper,
        }
    }
}

fn setup_energy_factory<EnergyFactoryObjBuilder>(
    b_mock: &mut BlockchainStateWrapper,
    owner: &Address,
    energy_factory_builder: EnergyFactoryObjBuilder,
) -> ContractObjWrapper<energy_factory::ContractObj<DebugApi>, EnergyFactoryObjBuilder>
where
    EnergyFactoryObjBuilder: 'static + Copy + Fn() -> energy_factory::ContractObj<DebugApi>,
{
    let rust_zero = rust_biguint!(0u64);
    let energy_factory_wrapper = b_mock.create_sc_account(
        &rust_zero,
        Some(owner),
        energy_factory_builder,
        "energy factory",
    );

    b_mock
        .execute_tx(owner, &energy_factory_wrapper, &rust_zero, |sc| {
            let mut lock_options = MultiValueEncoded::new();
            for (option, penalty) in LOCK_OPTIONS.iter().zip(PENALTY_PERCENTAGES.iter()) {
                lock_options.push((*option, *penalty).into());
            }

            sc.init(
                managed_token_id!(BASE_ASSET_TOKEN_ID),
                managed_token_id!(LEGACY_LOCKED_TOKEN_ID),
                managed_address!(energy_factory_wrapper.address_ref()),
                0,
                lock_options,
            );

            sc.locked_token()
                .set_token_id(managed_token_id!(LOCKED_TOKEN_ID));
            sc.set_paused(false);
        })
        .assert_ok();

    b_mock.set_esdt_local_roles(
        energy_factory_wrapper.address_ref(),
        BASE_ASSET_TOKEN_ID,
        ESDT_ROLES,
    );
    b_mock.set_esdt_local_roles(
        energy_factory_wrapper.address_ref(),
        LOCKED_TOKEN_ID,
        SFT_WITH_TRANSFER_ROLES,
    );
    b_mock.set_esdt_local_roles(
        energy_factory_wrapper.address_ref(),
        LEGACY_LOCKED_TOKEN_ID,
        &[EsdtLocalRole::NftBurn],
    );

    energy_factory_wrapper
}

fn setup_fees_collector<FeesCollectorObjBuilder>(
    b_mock: &mut BlockchainStateWrapper,
    owner: &Address,
    energy_factory_address: &Address,
    fees_collector_builder: FeesCollectorObjBuilder,
) -> ContractObjWrapper<fees_collector::ContractObj<DebugApi>, FeesCollectorObjBuilder>
where
    FeesCollectorObjBuilder: 'static + Copy + Fn() -> fees_collector::ContractObj<DebugApi>,
{
    let rust_zero = rust_biguint!(0u64);
    let fees_collector_wrapper = b_mock.create_sc_account(
        &rust_zero,
        Some(owner),
        fees_collector_builder,
        "fees collector",
    );

    b_mock
        .execute_tx(owner, &fees_collector_wrapper, &rust_zero, |sc| {
            sc.init(
                managed_token_id!(LOCKED_TOKEN_ID),
                managed_address!(energy_factory_address),
            );
            sc.set_paused(false);
        })
        .assert_ok();

    fees_collector_wrapper
}

fn setup_locked_token_wrapper<EnergyFactoryObjBuilder, LockedTokenWrapperObjBuilder>(
    b_mock: &mut BlockchainStateWrapper,
    owner: &Address,
    energy_factory_wrapper: &ContractObjWrapper<
        energy_factory::ContractObj<DebugApi>,
        EnergyFactoryObjBuilder,
    >,
    locked_token_wrapper_builder: LockedTokenWrapperObjBuilder,
) -> ContractObjWrapper<locked_token_wrapper::ContractObj<DebugApi>, LockedTokenWrapperObjBuilder>
where
    LockedTokenWrapperObjBuilder:
        'static + Copy + Fn() -> locked_token_wrapper::ContractObj<DebugApi>,
    EnergyFactoryObjBuilder: 'static + Copy + Fn() -> energy_factory::ContractObj<DebugApi>,
{
    let rust_zero = rust_biguint!(0u64);
    let locked_token_wrapper = b_mock.create_sc_account(
        &rust_zero,
        Some(owner),
        locked_token_wrapper_builder,
        "locked token wrapper",
    );

    b_mock
        .execute_tx(owner, &locked_token_wrapper, &rust_zero, |sc| {
            sc.init(managed_address!(energy_factory_wrapper.address_ref()));

            sc.wrapped_token()
                .set_token_id(managed_token_id!(WRAPPED_LOCKED_TOKEN_ID));
        })
        .assert_ok();

    b_mock
        .execute_tx(owner, energy_factory_wrapper, &rust_zero, |sc| {
            sc.add_sc_address_to_whitelist(managed_address!(locked_token_wrapper.address_ref()));
            let mut address_to_whitelist = MultiValueEncoded::new();
            address_to_whitelist.push(managed_address!(locked_token_wrapper.address_ref()));
            sc.add_to_token_transfer_whitelist(address_to_whitelist);
        })
        .assert_ok();

    b_mock.set_esdt_local_roles(
        locked_token_wrapper.address_ref(),
        WRAPPED_LOCKED_TOKEN_ID,
        SFT_WITH_TRANSFER_ROLES,
    );

    locked_token_wrapper
}

fn setup_pair<PairObjBuilder>(
    b_mock: &mut BlockchainStateWrapper,
    owner: &Address,
    pair_builder: PairObjBuilder,
) -> ContractObjWrapper<pair::ContractObj<DebugApi>, PairObjBuilder>
where
    PairObjBuilder: 'static + Copy + Fn() -> pair::ContractObj<DebugApi>,
{
    let rust_zero = rust_biguint!(0u64);
    let pair_wrapper = b_mock.create_sc_account(&rust_zero, Some(owner), pair_builder, "pair");

    b_mock
        .execute_tx(owner, &pair_wrapper, &rust_zero, |sc| {
            sc.init(
                managed_token_id!(BASE_FARM_STAKING_TOKEN_ID),
                managed_token_id!(OTHER_FARM_STAKING_TOKEN_ID),
                managed_address!(owner),
                managed_address!(owner),
                0,
                0,
                ManagedAddress::<DebugApi>::zero(),
                MultiValueEncoded::<DebugApi, ManagedAddress<DebugApi>>::new(),
            );

            sc.lp_token_identifier()
                .set(&managed_token_id!(FARMING_TOKEN_ID));
            sc.state().set(State::Active);
        })
        .assert_ok();

    b_mock.set_esdt_local_roles(pair_wrapper.address_ref(), FARMING_TOKEN_ID, ESDT_ROLES);

    pair_wrapper
}

fn setup_farm<FarmObjBuilder, EnergyFactoryObjBuilder>(
    b_mock: &mut BlockchainStateWrapper,
    owner: &Address,
    pair_address: &Address,
    energy_factory_wrapper: &ContractObjWrapper<
        energy_factory::ContractObj<DebugApi>,
        EnergyFactoryObjBuilder,
    >,
    farm_builder: FarmObjBuilder,
) -> ContractObjWrapper<farm_with_locked_rewards::ContractObj<DebugApi>, FarmObjBuilder>
where
    FarmObjBuilder: 'static + Copy + Fn() -> farm_with_locked_rewards::ContractObj<DebugApi>,
    EnergyFactoryObjBuilder: 'static + Copy + Fn() -> energy_factory::ContractObj<DebugApi>,
{
    let rust_zero = rust_biguint!(0u64);
    let farm_wrapper = b_mock.create_sc_account(
        &rust_zero,
        Some(owner),
        farm_builder,
        "farm with locked rewards",
    );

    b_mock
        .execute_tx(owner, &farm_wrapper, &rust_zero, |sc| {
            let mut admins = MultiValueEncoded::new();
            admins.push(managed_address!(owner));
            sc.init(
                managed_token_id!(BASE_ASSET_TOKEN_ID),
                managed_token_id!(FARMING_TOKEN_ID),
                managed_biguint!(DIVISION_SAFETY_CONSTANT),
                managed_address!(pair_address), // not important at this moment
                managed_address!(owner),
                admins,
            );
            sc.farm_token()
                .set_token_id(managed_token_id!(FARM_TOKEN_ID));
            sc.set_locking_sc_address(managed_address!(energy_factory_wrapper.address_ref()));
            sc.set_lock_epochs(*LOCK_OPTIONS.last().unwrap());
            sc.set_minimum_farming_epochs(UNBOND_PERIOD);

            sc.per_block_reward_amount()
                .set(&managed_biguint!(PER_BLOCK_REWARD_AMOUNT));

            sc.state().set(State::Active);
            sc.produce_rewards_enabled().set(true);
            sc.set_energy_factory_address(managed_address!(energy_factory_wrapper.address_ref()));

            sc.set_boosted_yields_factors(
                managed_biguint!(USER_REWARDS_BASE_CONST),
                managed_biguint!(USER_REWARDS_ENERGY_CONST),
                managed_biguint!(USER_REWARDS_FARM_CONST),
                managed_biguint!(MIN_ENERGY_AMOUNT_FOR_BOOSTED_YIELDS),
                managed_biguint!(MIN_FARM_AMOUNT_FOR_BOOSTED_YIELDS),
            );
            sc.set_boosted_yields_rewards_percentage(BOOSTED_YIELDS_PERCENTAGE);
        })
        .assert_ok();

    b_mock
        .execute_tx(owner, energy_factory_wrapper, &rust_zero, |sc| {
            sc.sc_whitelist_addresses()
                .add(&managed_address!(farm_wrapper.address_ref()));
        })
        .assert_ok();

    let farm_token_roles = [
        EsdtLocalRole::NftCreate,
        EsdtLocalRole::NftAddQuantity,
        EsdtLocalRole::NftBurn,
    ];
    b_mock.set_esdt_local_roles(
        farm_wrapper.address_ref(),
        FARM_TOKEN_ID,
        &farm_token_roles[..],
    );

    b_mock.set_esdt_local_roles(
        energy_factory_wrapper.address_ref(),
        LOCKED_TOKEN_ID,
        SFT_WITH_TRANSFER_ROLES,
    );

    farm_wrapper
}

fn setup_farm_staking<FarmStakingObjBuilder>(
    b_mock: &mut BlockchainStateWrapper,
    owner: &Address,
    farm_staking_builder: FarmStakingObjBuilder,
) -> ContractObjWrapper<farm_staking::ContractObj<DebugApi>, FarmStakingObjBuilder>
where
    FarmStakingObjBuilder: 'static + Copy + Fn() -> farm_staking::ContractObj<DebugApi>,
{
    let rust_zero = rust_biguint!(0u64);
    let farm_staking_wrapper = b_mock.create_sc_account(
        &rust_zero,
        Some(owner),
        farm_staking_builder,
        "farm staking",
    );

    b_mock
        .execute_tx(owner, &farm_staking_wrapper, &rust_zero, |sc| {
            sc.init(
                managed_token_id!(BASE_FARM_STAKING_TOKEN_ID),
                managed_biguint!(DIVISION_SAFETY_CONSTANT),
                managed_biguint!(MAX_APR_PERCENTAGE),
                UNBOND_PERIOD,
                ManagedAddress::<DebugApi>::zero(),
                MultiValueEncoded::new(),
            );

            sc.farm_token()
                .set_token_id(managed_token_id!(STAKING_FARM_TOKEN_ID));

            sc.per_block_reward_amount()
                .set(&managed_biguint!(PER_BLOCK_REWARD_AMOUNT));

            sc.state().set(State::Active);
            sc.produce_rewards_enabled().set(true);
        })
        .assert_ok();

    b_mock.set_esdt_local_roles(
        farm_staking_wrapper.address_ref(),
        STAKING_FARM_TOKEN_ID,
        SFT_WITH_TRANSFER_ROLES,
    );

    let farming_token_roles = [EsdtLocalRole::Burn];
    b_mock.set_esdt_local_roles(
        farm_staking_wrapper.address_ref(),
        BASE_FARM_STAKING_TOKEN_ID,
        &farming_token_roles[..],
    );

    b_mock.set_esdt_balance(
        owner,
        BASE_FARM_STAKING_TOKEN_ID,
        &TOTAL_REWARDS_AMOUNT.into(),
    );
    b_mock
        .execute_esdt_transfer(
            owner,
            &farm_staking_wrapper,
            BASE_FARM_STAKING_TOKEN_ID,
            0,
            &TOTAL_REWARDS_AMOUNT.into(),
            |sc| {
                sc.top_up_rewards();
            },
        )
        .assert_ok();

    farm_staking_wrapper
}

fn setup_farm_staking_proxy<FarmStakingProxyObjBuilder, FarmObjBuilder, FarmStakingObjBuilder>(
    b_mock: &mut BlockchainStateWrapper,
    owner: &Address,
    energy_factory_address: &Address,
    pair_address: &Address,
    lp_farm_wrapper: &ContractObjWrapper<
        farm_with_locked_rewards::ContractObj<DebugApi>,
        FarmObjBuilder,
    >,
    farm_staking_wrapper: &ContractObjWrapper<
        farm_staking::ContractObj<DebugApi>,
        FarmStakingObjBuilder,
    >,
    farm_staking_proxy_builder: FarmStakingProxyObjBuilder,
) -> ContractObjWrapper<farm_staking_proxy::ContractObj<DebugApi>, FarmStakingProxyObjBuilder>
where
    FarmStakingProxyObjBuilder: 'static + Copy + Fn() -> farm_staking_proxy::ContractObj<DebugApi>,
    FarmObjBuilder: 'static + Copy + Fn() -> farm_with_locked_rewards::ContractObj<DebugApi>,
    FarmStakingObjBuilder: 'static + Copy + Fn() -> farm_staking::ContractObj<DebugApi>,
{
    let rust_zero = rust_biguint!(0u64);
    let farm_staking_proxy_wrapper = b_mock.create_sc_account(
        &rust_zero,
        Some(owner),
        farm_staking_proxy_builder,
        "farm staking proxy",
    );

    b_mock
        .execute_tx(owner, &farm_staking_proxy_wrapper, &rust_zero, |sc| {
            sc.init(
                managed_address!(energy_factory_address),
                managed_address!(lp_farm_wrapper.address_ref()),
                managed_address!(farm_staking_wrapper.address_ref()),
                managed_address!(pair_address),
                managed_token_id!(BASE_FARM_STAKING_TOKEN_ID),
                managed_token_id!(FARM_TOKEN_ID),
                managed_token_id!(STAKING_FARM_TOKEN_ID),
                managed_token_id!(FARMING_TOKEN_ID),
            );

            sc.dual_yield_token()
                .set_token_id(managed_token_id!(DUAL_YIELD_TOKEN_ID));
        })
        .assert_ok();

    b_mock.set_esdt_local_roles(
        farm_staking_proxy_wrapper.address_ref(),
        DUAL_YIELD_TOKEN_ID,
        SFT_WITH_TRANSFER_ROLES,
    );

    b_mock
        .execute_tx(owner, farm_staking_wrapper, &rust_zero, |sc| {
            sc.add_sc_address_to_whitelist(managed_address!(
                farm_staking_proxy_wrapper.address_ref()
            ));
        })
        .assert_ok();

    b_mock
        .execute_tx(owner, lp_farm_wrapper, &rust_zero, |sc| {
            sc.add_sc_address_to_whitelist(managed_address!(
                farm_staking_proxy_wrapper.address_ref()
            ));
        })
        .assert_ok();

    farm_staking_proxy_wrapper
}
