#![allow(deprecated)]

use crate::pair_setup::PairSetup;

use super::metastaking_setup::setup_metastaking;
use auto_farm::whitelists::{
    farms_whitelist::FarmsWhitelistModule, metastaking_whitelist::MetastakingWhitelistModule,
};
use auto_pos_creator::{configs::pairs_config::PairsConfigModule, AutoPosCreator};
use multiversx_sc::types::{ManagedVec, MultiValueEncoded};
use multiversx_sc_scenario::{
    managed_address, managed_biguint, managed_token_id, rust_biguint,
    testing_framework::ContractObjWrapper, DebugApi,
};
use sc_whitelist_module::SCWhitelistModule;
use tests_common::{
    farm_staking_setup::{setup_farm_staking, STAKING_FARM_TOKEN_ID},
    farm_with_locked_rewards_setup::{FarmSetup, FARMING_TOKEN_ID, FARM_TOKEN_ID},
};

use pair::safe_price::SafePriceModule;

pub static TOKEN_IDS: &[&[u8]] = &[b"FIRST-123456", b"SECOND-123456", b"THIRD-123456"];
pub static LP_TOKEN_IDS: &[&[u8]] = &[FARMING_TOKEN_ID[0], FARMING_TOKEN_ID[1], b"LPTHIRD-123456"];
pub static WEGLD_TOKEN_ID: &[u8] = b"WEGLD-123456";

pub struct PosCreatorSetup<
    FarmBuilder,
    EnergyFactoryBuilder,
    PairBuilder,
    FarmStakingBuilder,
    MetastakingBuilder,
    PosCreatorBuilder,
> where
    FarmBuilder: 'static + Copy + Fn() -> farm_with_locked_rewards::ContractObj<DebugApi>,
    EnergyFactoryBuilder: 'static + Copy + Fn() -> energy_factory::ContractObj<DebugApi>,
    PairBuilder: 'static + Copy + Fn() -> pair::ContractObj<DebugApi>,
    FarmStakingBuilder: 'static + Copy + Fn() -> farm_staking::ContractObj<DebugApi>,
    MetastakingBuilder: 'static + Copy + Fn() -> farm_staking_proxy::ContractObj<DebugApi>,
    PosCreatorBuilder: 'static + Copy + Fn() -> auto_pos_creator::ContractObj<DebugApi>,
{
    pub farm_setup: FarmSetup<FarmBuilder, EnergyFactoryBuilder>,
    pub pair_setups: Vec<PairSetup<PairBuilder>>,
    pub fs_wrapper: ContractObjWrapper<farm_staking::ContractObj<DebugApi>, FarmStakingBuilder>,
    pub ms_wrapper:
        ContractObjWrapper<farm_staking_proxy::ContractObj<DebugApi>, MetastakingBuilder>,
    pub pos_creator_wrapper:
        ContractObjWrapper<auto_pos_creator::ContractObj<DebugApi>, PosCreatorBuilder>,
}

impl<
        FarmBuilder,
        EnergyFactoryBuilder,
        PairBuilder,
        FarmStakingBuilder,
        MetastakingBuilder,
        PosCreatorBuilder,
    >
    PosCreatorSetup<
        FarmBuilder,
        EnergyFactoryBuilder,
        PairBuilder,
        FarmStakingBuilder,
        MetastakingBuilder,
        PosCreatorBuilder,
    >
where
    FarmBuilder: 'static + Copy + Fn() -> farm_with_locked_rewards::ContractObj<DebugApi>,
    EnergyFactoryBuilder: 'static + Copy + Fn() -> energy_factory::ContractObj<DebugApi>,
    PairBuilder: 'static + Copy + Fn() -> pair::ContractObj<DebugApi>,
    FarmStakingBuilder: 'static + Copy + Fn() -> farm_staking::ContractObj<DebugApi>,
    MetastakingBuilder: 'static + Copy + Fn() -> farm_staking_proxy::ContractObj<DebugApi>,
    PosCreatorBuilder: 'static + Copy + Fn() -> auto_pos_creator::ContractObj<DebugApi>,
{
    // Pairs setup:
    // 3 pools (A, B), (A, C), (B, C),
    // A:B 1:2
    // A:C 1:6
    // B:C 1:3

    // Pools: (B = billion)
    // (A, B) => (1B, 2B)
    // (A, C) => (1B, 6B)
    // (B, C) => (1B, 3B)
    //
    // A_total = 2B
    // B_total = 3B
    // C_total = 9B
    pub fn new(
        farm_builder: FarmBuilder,
        energy_factory_builder: EnergyFactoryBuilder,
        pair_builder: PairBuilder,
        farm_staking_builder: FarmStakingBuilder,
        metastaking_builder: MetastakingBuilder,
        pos_creator_builder: PosCreatorBuilder,
    ) -> Self {
        let farm_setup = FarmSetup::new(farm_builder, energy_factory_builder);
        let b_mock = farm_setup.b_mock.clone();

        // undo the set_balance FarmSetup does for Farming tokens
        b_mock.borrow_mut().set_esdt_balance(
            &farm_setup.first_user,
            LP_TOKEN_IDS[0],
            &rust_biguint!(0),
        );
        b_mock.borrow_mut().set_esdt_balance(
            &farm_setup.first_user,
            LP_TOKEN_IDS[1],
            &rust_biguint!(0),
        );

        let owner = farm_setup.owner.clone();
        let mut first_pair_setup = PairSetup::new(
            b_mock.clone(),
            pair_builder,
            &owner,
            TOKEN_IDS[0],
            TOKEN_IDS[1],
            LP_TOKEN_IDS[0],
        );
        let mut second_pair_setup = PairSetup::new(
            b_mock.clone(),
            pair_builder,
            &owner,
            TOKEN_IDS[0],
            TOKEN_IDS[2],
            LP_TOKEN_IDS[1],
        );
        let mut third_pair_setup = PairSetup::new(
            b_mock.clone(),
            pair_builder,
            &owner,
            TOKEN_IDS[1],
            TOKEN_IDS[2],
            LP_TOKEN_IDS[2],
        );

        let first_token_total_amount = 2_000_000_000u64;
        let second_token_total_amount = 3_000_000_000u64;
        let third_token_total_amount = 9_000_000_000u64;
        b_mock.borrow_mut().set_esdt_balance(
            &owner,
            TOKEN_IDS[0],
            &rust_biguint!(first_token_total_amount),
        );
        b_mock.borrow_mut().set_esdt_balance(
            &owner,
            TOKEN_IDS[1],
            &rust_biguint!(second_token_total_amount),
        );
        b_mock.borrow_mut().set_esdt_balance(
            &owner,
            TOKEN_IDS[2],
            &rust_biguint!(third_token_total_amount),
        );

        let mut block_round: u64 = 1;
        b_mock.borrow_mut().set_block_round(block_round);

        // add initial liquidity
        first_pair_setup.add_liquidity(&owner, 1_000_000_000, 2_000_000_000);
        second_pair_setup.add_liquidity(&owner, 1_000_000_000, 6_000_000_000);
        third_pair_setup.add_liquidity(&owner, 1_000_000_000, 3_000_000_000);

        // setup price observations
        for _i in 1usize..=20 {
            block_round += 1;
            b_mock.borrow_mut().set_block_round(block_round);

            b_mock
                .borrow_mut()
                .execute_tx(
                    &owner,
                    &first_pair_setup.pair_wrapper,
                    &rust_biguint!(0),
                    |sc| {
                        sc.update_safe_price(
                            &managed_biguint!(1_000_000_000),
                            &managed_biguint!(2_000_000_000),
                        );
                    },
                )
                .assert_ok();

            b_mock
                .borrow_mut()
                .execute_tx(
                    &owner,
                    &second_pair_setup.pair_wrapper,
                    &rust_biguint!(0),
                    |sc| {
                        sc.update_safe_price(
                            &managed_biguint!(1_000_000_000),
                            &managed_biguint!(6_000_000_000),
                        );
                    },
                )
                .assert_ok();

            b_mock
                .borrow_mut()
                .execute_tx(
                    &owner,
                    &third_pair_setup.pair_wrapper,
                    &rust_biguint!(0),
                    |sc| {
                        sc.update_safe_price(
                            &managed_biguint!(1_000_000_000),
                            &managed_biguint!(3_000_000_000),
                        );
                    },
                )
                .assert_ok();
        }

        // setup farm staking
        let fs_wrapper = setup_farm_staking(
            &mut b_mock.borrow_mut(),
            farm_staking_builder,
            TOKEN_IDS[0],
            TOKEN_IDS[0],
        );

        // setup metastaking
        let ms_wrapper = setup_metastaking(
            &mut b_mock.borrow_mut(),
            metastaking_builder,
            &owner,
            farm_setup.energy_factory_wrapper.address_ref(),
            farm_setup.farm_wrappers[0].address_ref(),
            fs_wrapper.address_ref(),
            first_pair_setup.pair_wrapper.address_ref(),
            TOKEN_IDS[0],
            FARM_TOKEN_ID[0],
            STAKING_FARM_TOKEN_ID,
            LP_TOKEN_IDS[0],
        );

        // setup auto pos creator sc
        let pos_creator_wrapper = b_mock.borrow_mut().create_sc_account(
            &rust_biguint!(0),
            Some(&owner),
            pos_creator_builder,
            "auto pos creator",
        );

        b_mock
            .borrow_mut()
            .execute_tx(&owner, &pos_creator_wrapper, &rust_biguint!(0), |sc| {
                sc.init(
                    managed_address!(pos_creator_wrapper.address_ref()), // unused
                    managed_token_id!(WEGLD_TOKEN_ID),
                );

                let mut farms = MultiValueEncoded::new();
                farms.push(managed_address!(farm_setup.farm_wrappers[0].address_ref()));
                farms.push(managed_address!(farm_setup.farm_wrappers[1].address_ref()));
                farms.push(managed_address!(fs_wrapper.address_ref()));

                sc.add_farms(farms);
                sc.add_metastaking_scs(
                    ManagedVec::from_single_item(managed_address!(ms_wrapper.address_ref())).into(),
                );

                let mut pairs = MultiValueEncoded::new();
                pairs.push(managed_address!(first_pair_setup
                    .pair_wrapper
                    .address_ref()));
                pairs.push(managed_address!(second_pair_setup
                    .pair_wrapper
                    .address_ref()));
                pairs.push(managed_address!(third_pair_setup
                    .pair_wrapper
                    .address_ref()));

                sc.add_pairs_to_whitelist(pairs);
            })
            .assert_ok();

        // add auto pos creator SC to metastaking whitelist
        b_mock
            .borrow_mut()
            .execute_tx(&owner, &ms_wrapper, &rust_biguint!(0), |sc| {
                sc.sc_whitelist_addresses()
                    .add(&managed_address!(pos_creator_wrapper.address_ref()));
            })
            .assert_ok();

        // add auto pos and metastaking to farm-staking whitelist
        b_mock
            .borrow_mut()
            .execute_tx(&owner, &fs_wrapper, &rust_biguint!(0), |sc| {
                sc.sc_whitelist_addresses()
                    .add(&managed_address!(pos_creator_wrapper.address_ref()));
                sc.sc_whitelist_addresses()
                    .add(&managed_address!(ms_wrapper.address_ref()));
            })
            .assert_ok();

        // add auto pos SC and metastaking SC to LP farm whitelist
        b_mock
            .borrow_mut()
            .execute_tx(
                &owner,
                &farm_setup.farm_wrappers[0],
                &rust_biguint!(0),
                |sc| {
                    sc.sc_whitelist_addresses()
                        .add(&managed_address!(pos_creator_wrapper.address_ref()));
                    sc.sc_whitelist_addresses()
                        .add(&managed_address!(ms_wrapper.address_ref()));
                },
            )
            .assert_ok();

        b_mock
            .borrow_mut()
            .execute_tx(
                &owner,
                &farm_setup.farm_wrappers[1],
                &rust_biguint!(0),
                |sc| {
                    sc.sc_whitelist_addresses()
                        .add(&managed_address!(pos_creator_wrapper.address_ref()));
                },
            )
            .assert_ok();

        let pair_setups = vec![first_pair_setup, second_pair_setup, third_pair_setup];

        PosCreatorSetup {
            farm_setup,
            pair_setups,
            fs_wrapper,
            ms_wrapper,
            pos_creator_wrapper,
        }
    }
}
