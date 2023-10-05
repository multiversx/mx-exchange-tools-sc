#![allow(deprecated)]

use composable_tasks::task_call::{TaskCall, TaskType};
use composable_tasks_setup::ComposableTasksSetup;
use multiversx_sc::types::{ManagedVec, MultiValueEncoded};
use multiversx_sc_scenario::{api::StaticApi, *};
use wegld_swap_setup::WEGLD_TOKEN_ID;

type ComposableTasksContract = ContractInfo<composable_tasks::Proxy<StaticApi>>;
type PairContract = ContractInfo<pair::Proxy<StaticApi>>;
type WegldSwap = ContractInfo<multiversx_wegld_swap_sc::Proxy<StaticApi>>;

// fn world() -> ScenarioWorld {
//     let mut blockchain = ScenarioWorld::new();
//     blockchain.set_current_dir_from_workspace("contracts/examples/empty");

//     blockchain.register_contract("file:output/composable-tasks.wasm", composable_tasks::ContractBuilder);
//     blockchain
// }

// struct ComposableTasksTestState {
//     world: ScenarioWorld,
//     owner_address: Address,
//     composable_tasks_contract: ComposableTasksContract,
//     pair_address: PairContract,
//     wegld_swap: WegldSwap,
// }

pub mod composable_tasks_setup;
pub mod pair_setup;
pub mod wegld_swap_setup;

#[test]
fn full_composable_tasks_setup_test() {
    let _ = ComposableTasksSetup::new(
        pair::contract_obj,
        multiversx_wegld_swap_sc::contract_obj,
        composable_tasks::contract_obj,
    );
}

#[test]
fn unwrap_wrap_test() {
    let composable_tasks_setup = ComposableTasksSetup::new(
        pair::contract_obj,
        multiversx_wegld_swap_sc::contract_obj,
        composable_tasks::contract_obj,
    );

    let b_mock = composable_tasks_setup.b_mock;
    let first_user_addr = composable_tasks_setup.first_user;
    let second_user_addr = composable_tasks_setup.second_user;

    let user_first_token_balance = 200_000_000u64;

    b_mock.borrow_mut().set_esdt_balance(
        &first_user_addr,
        WEGLD_TOKEN_ID,
        &rust_biguint!(user_first_token_balance),
    );

    b_mock
        .borrow_mut()
        .execute_esdt_transfer(
            &first_user_addr,
            &composable_tasks_setup.ct_wrapper,
            WEGLD_TOKEN_ID,
            0,
            &rust_biguint!(user_first_token_balance),
            |sc| {
                let no_args = ManagedVec::new();
                let mut tasks = MultiValueEncoded::new();
                tasks.push((TaskType::UnwrapEgld, no_args).into());

                let _ = sc.compose_tasks(managed_address!(&second_user_addr), tasks);
            },
        )
        .assert_ok();
}

// #[test]
// fn deploy_scs_test() {
//     world().run("scenarios/empty.scen.json");
// }
