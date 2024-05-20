#![allow(non_snake_case)]

mod composable_tasks_config;

use composable_tasks::composable_tasks_proxy::{self, TaskType};
use multiversx_sc_snippets::{imports::*, sdk};
use serde::{Deserialize, Serialize};
use std::{
    io::{Read, Write},
    path::Path,
};
use composable_tasks_config::Config;

const GATEWAY: &str = sdk::blockchain::DEVNET_GATEWAY;
const STATE_FILE: &str = "state.toml";

const HALF_UNIT: u64 = 500000000000000000; // 0.5 
const ONE_UNIT: u64 = 1000000000000000000; // 0.5 
const MILLION: u64 = 1_000_000; 

pub static WEGLD_TOKEN_ID: &[u8] = b"WEGLD-a28c59";
pub static MEX_TOKEN_ID: &[u8] = b"MEX-a659d0";
pub static ONE_TOKEN_ID: &[u8] = b"ONE-83a7c0";
pub static USDC_TOKEN_ID: &[u8] = b"USDC-350c4e";
pub static UTK_TOKEN_ID: &[u8] = b"UTK-14d57d";


pub const SWAP_TOKENS_FIXED_INPUT_FUNC_NAME: &[u8] = b"swapTokensFixedInput";
pub const SWAP_TOKENS_FIXED_OUTPUT_FUNC_NAME: &[u8] = b"swapTokensFixedOutput";



#[tokio::main]
async fn main() {
    env_logger::init();

    let mut args = std::env::args();
    let _ = args.next();
    let cmd = args.next().expect("at least one argument required");
    let mut interact = ContractInteract::new().await;
    match cmd.as_str() {
        "deploy" => interact.deploy().await,
        "setWrapEgldAddr" => interact.set_wrap_egld_address().await,
        "setRouterAddr" => interact.set_router_address().await,
        "wrap" => interact.wrap_test().await,
        "unwrap" => interact.unwrap_test().await,
        "wrapSwap" => interact.wrap_swap_test().await,
        "wrapSwapOutput" => interact.wrap_swap_fixed_output_test().await,
        "swapUnwrap" => interact.swap_unwrap_test().await,
        "swapFailMultipleInputTokens" => interact.swap_fail_sending_multiple_tokens_test().await,
        "wrapSwapFailLowOutputAmount" => interact.wrap_swap_fail_low_output_amount_test().await,
        "swapSendUnwrap" => interact.swap_send_unwrap_test().await,
        "multipleSwapsFixedOutput" => interact.multiple_swap_fixed_output_test().await,
        "mixingAllActions" => interact.mixing_all_actions_test().await,
        "routerMultiPairSwaps" => interact.router_multi_pair_swaps().await,
        _ => panic!("unknown command: {}", &cmd),
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct State {
    contract_address: Option<Bech32Address>,
}

impl State {
    // Deserializes state from file
    pub fn load_state() -> Self {
        if Path::new(STATE_FILE).exists() {
            let mut file = std::fs::File::open(STATE_FILE).unwrap();
            let mut content = String::new();
            file.read_to_string(&mut content).unwrap();
            toml::from_str(&content).unwrap()
        } else {
            Self::default()
        }
    }

    /// Sets the contract address
    pub fn set_address(&mut self, address: Bech32Address) {
        self.contract_address = Some(address);
    }

    /// Returns the contract address
    pub fn current_address(&self) -> &Bech32Address {
        self.contract_address
            .as_ref()
            .expect("no known contract, deploy first")
    }
}

impl Drop for State {
    // Serializes state to file
    fn drop(&mut self) {
        let mut file = std::fs::File::create(STATE_FILE).unwrap();
        file.write_all(toml::to_string(self).unwrap().as_bytes())
            .unwrap();
    }
}

struct ContractInteract {
    interactor: Interactor,
    wallet_address: Address,
    contract_code: BytesValue,
    config: Config,
    state: State,
}

impl ContractInteract {
    async fn new() -> Self {
        let mut interactor = Interactor::new(GATEWAY).await;
        let wallet_address = interactor.register_wallet(test_wallets::alice());

        let contract_code = BytesValue::interpret_from(
            "mxsc:../output/composable-tasks.mxsc.json",
            &InterpreterContext::default(),
        );


        ContractInteract {
            interactor,
            wallet_address,
            contract_code,
            config: Config::load_config(),
            state: State::load_state(),
        }
    }

    async fn deploy(&mut self) {
        let new_address = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .typed(composable_tasks_proxy::ComposableTasksContractProxy)
            .init()
            .code(&self.contract_code)
            .gas(100_000_000)
            .returns(ReturnsNewAddress)
            .prepare_async()
            .run()
            .await;
        let new_address_bech32 = bech32::encode(&new_address);
        self.state.set_address(Bech32Address::from_bech32_string(
            new_address_bech32.clone(),
        ));

        println!("new address: {new_address_bech32}");
    }

    async fn set_wrap_egld_address(&mut self) {
        let wegld_address = &self.config.wegld_address;

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .typed(composable_tasks_proxy::ComposableTasksContractProxy)
            .set_wrap_egld_address(wegld_address)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn set_router_address(&mut self) {
        let router_address = &self.config.router_address;

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .typed(composable_tasks_proxy::ComposableTasksContractProxy)
            .set_router_address(router_address)
            .returns(ReturnsResult)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }


    async fn wrap_test(&mut self) {
        let token_nonce = 0u64;

        let min_expected_token_out = EgldOrEsdtTokenPayment::new(
            EgldOrEsdtTokenIdentifier::esdt(WEGLD_TOKEN_ID),
            0u64,
            BigUint::from(ONE_UNIT),
        );

        let no_args = ManagedVec::new();
        let mut tasks = MultiValueEncoded::new();
        tasks.push((TaskType::WrapEGLD, no_args).into());

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .typed(composable_tasks_proxy::ComposableTasksContractProxy)
            .compose_tasks(min_expected_token_out, tasks)
            .gas(50_000_000)
            .payment(EgldOrEsdtTokenPayment::new(
                EgldOrEsdtTokenIdentifier::egld(),
                token_nonce,
                BigUint::from(ONE_UNIT),
            ))
            .returns(ReturnsResult)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }


    async fn unwrap_test(&mut self) {
        let token_nonce = 0u64;

        let min_expected_token_out = EgldOrEsdtTokenPayment::new(
            EgldOrEsdtTokenIdentifier::egld(),
            0u64,
            BigUint::from(ONE_UNIT),
        );

        let no_args = ManagedVec::new();
        let mut tasks = MultiValueEncoded::new();
        tasks.push((TaskType::UnwrapEGLD, no_args).into());

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .typed(composable_tasks_proxy::ComposableTasksContractProxy)
            .compose_tasks(min_expected_token_out, tasks)
            .gas(10_000_000)
            .payment(EgldOrEsdtTokenPayment::new(
                EgldOrEsdtTokenIdentifier::esdt(WEGLD_TOKEN_ID),
                token_nonce,
                BigUint::from(ONE_UNIT),
            ))
            .returns(ReturnsResult)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn wrap_swap_test(&mut self) {
        let token_nonce = 0u64;
        let token_amount = BigUint::<StaticApi>::from(HALF_UNIT);

        let min_expected_token_out = EgldOrEsdtTokenPayment::new(
            EgldOrEsdtTokenIdentifier::esdt(MEX_TOKEN_ID),
            0u64,
            BigUint::from(HALF_UNIT),
        );

        let no_args = ManagedVec::new();
        let mut swap_args = ManagedVec::new();
        swap_args.push(managed_buffer!(SWAP_TOKENS_FIXED_INPUT_FUNC_NAME));
        swap_args.push(managed_buffer!(MEX_TOKEN_ID));
        swap_args.push(managed_buffer!(b"1"));

        let mut tasks = MultiValueEncoded::new();
        tasks.push((TaskType::WrapEGLD, no_args).into());
        tasks.push((TaskType::Swap, swap_args).into());

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .typed(composable_tasks_proxy::ComposableTasksContractProxy)
            .compose_tasks(min_expected_token_out, tasks)
            .gas(50_000_000)
            .payment(EgldOrEsdtTokenPayment::new(
                EgldOrEsdtTokenIdentifier::egld(),
                token_nonce,
                token_amount,
            ))
            .returns(ReturnsResult)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn wrap_swap_fixed_output_test(&mut self) {
        let token_nonce = 0u64;
        let token_amount = BigUint::<StaticApi>::from(HALF_UNIT);

        let min_expected_token_out = EgldOrEsdtTokenPayment::new(
            EgldOrEsdtTokenIdentifier::esdt(MEX_TOKEN_ID),
            0u64,
            BigUint::from(HALF_UNIT),
        );

        let no_args = ManagedVec::new();
        let mut swap_args = ManagedVec::new();
        swap_args.push(managed_buffer!(SWAP_TOKENS_FIXED_OUTPUT_FUNC_NAME));
        swap_args.push(managed_buffer!(MEX_TOKEN_ID));
        let one_mil = BigUint::from(ONE_UNIT) * MILLION;
        swap_args.push(one_mil.to_bytes_be_buffer());

        let mut tasks = MultiValueEncoded::new();
        tasks.push((TaskType::WrapEGLD, no_args).into());
        tasks.push((TaskType::Swap, swap_args).into());

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .typed(composable_tasks_proxy::ComposableTasksContractProxy)
            .compose_tasks(min_expected_token_out, tasks)
            .gas(50_000_000)
            .payment(EgldOrEsdtTokenPayment::new(
                EgldOrEsdtTokenIdentifier::egld(),
                token_nonce,
                token_amount,
            ))
            .returns(ReturnsResult)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }


    async fn swap_unwrap_test(&mut self) {
        let token_nonce = 0u64;

        let min_expected_token_out = EgldOrEsdtTokenPayment::new(
            EgldOrEsdtTokenIdentifier::egld(),
            0u64,
            BigUint::from(HALF_UNIT / 2),
        );

        let no_args = ManagedVec::new();
        let mut swap_args = ManagedVec::new();
        swap_args.push(managed_buffer!(SWAP_TOKENS_FIXED_INPUT_FUNC_NAME));
        swap_args.push(managed_buffer!(WEGLD_TOKEN_ID));
        swap_args.push(managed_buffer!(b"1"));

        let mut tasks = MultiValueEncoded::new();
        tasks.push((TaskType::Swap, swap_args).into());
        tasks.push((TaskType::UnwrapEGLD, no_args).into());

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .typed(composable_tasks_proxy::ComposableTasksContractProxy)
            .compose_tasks(min_expected_token_out, tasks)
            .gas(50_000_000)
            .payment(EgldOrEsdtTokenPayment::new(
                EgldOrEsdtTokenIdentifier::esdt(MEX_TOKEN_ID),
                token_nonce,
                BigUint::from(3 * ONE_UNIT) * BigUint::from(MILLION),
            ))
            .returns(ReturnsResult)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn swap_fail_sending_multiple_tokens_test(&mut self) {
        let token_nonce = 0u64;

        let min_expected_token_out = EgldOrEsdtTokenPayment::new(
            EgldOrEsdtTokenIdentifier::egld(),
            0u64,
            BigUint::from(HALF_UNIT / 2),
        );

        let no_args = ManagedVec::new();
        let mut swap_args = ManagedVec::new();
        swap_args.push(managed_buffer!(SWAP_TOKENS_FIXED_INPUT_FUNC_NAME));
        swap_args.push(managed_buffer!(WEGLD_TOKEN_ID));
        swap_args.push(managed_buffer!(b"1"));

        let mut tasks = MultiValueEncoded::new();
        tasks.push((TaskType::Swap, swap_args).into());
        tasks.push((TaskType::UnwrapEGLD, no_args).into());

        let mut multi_payments = MultiEsdtPayment::new();
        multi_payments.push(EsdtTokenPayment::new(
            TokenIdentifier::from(MEX_TOKEN_ID),
            token_nonce,
            BigUint::from(3 * ONE_UNIT) * BigUint::from(MILLION)));
            multi_payments.push(EsdtTokenPayment::new(
                    TokenIdentifier::from(ONE_TOKEN_ID),
                    token_nonce,
                    BigUint::from(3 * ONE_UNIT)));

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .typed(composable_tasks_proxy::ComposableTasksContractProxy)
            .compose_tasks(min_expected_token_out, tasks)
            .gas(50_000_000)
            .payment(multi_payments)
            .returns(ReturnsResult)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn wrap_swap_fail_low_output_amount_test(&mut self) {
        let token_nonce = 0u64;
        let token_amount = BigUint::<StaticApi>::from(HALF_UNIT);

        let min_expected_token_out = EgldOrEsdtTokenPayment::new(
            EgldOrEsdtTokenIdentifier::esdt(MEX_TOKEN_ID),
            0u64,
            BigUint::from(ONE_UNIT)* MILLION * 4u64,
        );

        let no_args = ManagedVec::new();
        let mut swap_args = ManagedVec::new();
        swap_args.push(managed_buffer!(SWAP_TOKENS_FIXED_INPUT_FUNC_NAME));
        swap_args.push(managed_buffer!(MEX_TOKEN_ID));
        swap_args.push(managed_buffer!(b"1"));

        let mut tasks = MultiValueEncoded::new();
        tasks.push((TaskType::WrapEGLD, no_args).into());
        tasks.push((TaskType::Swap, swap_args).into());

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .typed(composable_tasks_proxy::ComposableTasksContractProxy)
            .compose_tasks(min_expected_token_out, tasks)
            .gas(50_000_000)
            .payment(EgldOrEsdtTokenPayment::new(
                EgldOrEsdtTokenIdentifier::egld(),
                token_nonce,
                token_amount,
            ))
            .returns(ReturnsResult)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }


    async fn swap_send_unwrap_test(&mut self) {
        let token_nonce = 0u64;
        let second_address = &self.config.random_address;


        let min_expected_token_out = EgldOrEsdtTokenPayment::new(
            EgldOrEsdtTokenIdentifier::esdt(WEGLD_TOKEN_ID),
            0u64,
            BigUint::from(HALF_UNIT),
        );

        let no_args = ManagedVec::new();
        let mut swap_args = ManagedVec::new();
        swap_args.push(managed_buffer!(SWAP_TOKENS_FIXED_INPUT_FUNC_NAME));
        swap_args.push(managed_buffer!(WEGLD_TOKEN_ID));
        swap_args.push(managed_buffer!(b"1"));

        let mut tasks = MultiValueEncoded::new();
        tasks.push((TaskType::Swap, swap_args).into());

        let mut send_args = ManagedVec::new();
        send_args.push(managed_buffer!(second_address.as_address().as_bytes()));

        tasks.push((TaskType::SendEgldOrEsdt, send_args).into());
        tasks.push((TaskType::UnwrapEGLD, no_args).into()); // this should not be executed

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .typed(composable_tasks_proxy::ComposableTasksContractProxy)
            .compose_tasks(min_expected_token_out, tasks)
            .gas(50_000_000)
            .payment(EgldOrEsdtTokenPayment::new(
                EgldOrEsdtTokenIdentifier::esdt(MEX_TOKEN_ID),
                token_nonce,
                BigUint::from(5 * ONE_UNIT) * BigUint::from(MILLION),
            ))
            .returns(ReturnsResult)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }



    async fn multiple_swap_fixed_output_test(&mut self) {
        let min_expected_token_out = EgldOrEsdtTokenPayment::new(
            EgldOrEsdtTokenIdentifier::esdt(WEGLD_TOKEN_ID),
            0u64,
            BigUint::from(HALF_UNIT),
        );

        let mut tasks = MultiValueEncoded::new();
        let mut swap_args1 = ManagedVec::new();
        swap_args1.push(managed_buffer!(SWAP_TOKENS_FIXED_OUTPUT_FUNC_NAME));
        swap_args1.push(managed_buffer!(USDC_TOKEN_ID));
        let amount_38 = BigUint::from(30000000u64); // 38 units with 6 decimals
        swap_args1.push(amount_38.to_bytes_be_buffer()); 
        tasks.push((TaskType::Swap, swap_args1).into());

        let mut swap_args2 = ManagedVec::new();
        swap_args2.push(managed_buffer!(SWAP_TOKENS_FIXED_OUTPUT_FUNC_NAME));
        swap_args2.push(managed_buffer!(MEX_TOKEN_ID));
        let amount = BigUint::from(ONE_UNIT) * MILLION * 5u64;
        swap_args2.push(amount.to_bytes_be_buffer());
        tasks.push((TaskType::Swap, swap_args2).into());

        let mut swap_args3 = ManagedVec::new();
        swap_args3.push(managed_buffer!(SWAP_TOKENS_FIXED_OUTPUT_FUNC_NAME));
        swap_args3.push(managed_buffer!(WEGLD_TOKEN_ID));
        let one = BigUint::from(HALF_UNIT);
        swap_args3.push(one.to_bytes_be_buffer());
        tasks.push((TaskType::Swap, swap_args3).into());

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .typed(composable_tasks_proxy::ComposableTasksContractProxy)
            .compose_tasks(min_expected_token_out, tasks)
            .gas(100_000_000)
            .payment(EgldOrEsdtTokenPayment::new(
                EgldOrEsdtTokenIdentifier::esdt(WEGLD_TOKEN_ID),
                0u64,
                BigUint::from(ONE_UNIT),
            ))
            .returns(ReturnsResult)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn mixing_all_actions_test(&mut self) {
        let token_nonce = 0u64;
        let second_address = &self.config.random_address;
        let egld_mex_pair_address = &self.config.egld_mex_pair_address;

        let min_expected_token_out = EgldOrEsdtTokenPayment::new(
            EgldOrEsdtTokenIdentifier::esdt(WEGLD_TOKEN_ID),
            0u64,
            BigUint::from(HALF_UNIT),
        );

        let mut tasks = MultiValueEncoded::new();

        // Wrap EGLD
        let no_args = ManagedVec::new();
        tasks.push((TaskType::WrapEGLD, no_args).into());

        // Swap fixed output
        let mut swap_args_fixed_output = ManagedVec::new();
        swap_args_fixed_output.push(managed_buffer!(SWAP_TOKENS_FIXED_OUTPUT_FUNC_NAME));
        swap_args_fixed_output.push(managed_buffer!(USDC_TOKEN_ID));
        let amount_38 = BigUint::from(30000000u64); // 38 units with 6 decimals
        swap_args_fixed_output.push(amount_38.to_bytes_be_buffer()); 
        tasks.push((TaskType::Swap, swap_args_fixed_output).into());


        // Swap fixed input
        let mut swap_args = ManagedVec::new();
        swap_args.push(managed_buffer!(SWAP_TOKENS_FIXED_INPUT_FUNC_NAME));
        swap_args.push(managed_buffer!(MEX_TOKEN_ID));
        swap_args.push(managed_buffer!(b"1"));
        tasks.push((TaskType::Swap, swap_args).into());

        // Router swap
        let mut router_swap_args = ManagedVec::new();
        router_swap_args.push(managed_buffer!(egld_mex_pair_address.as_address().as_bytes()));
        router_swap_args.push(managed_buffer!(SWAP_TOKENS_FIXED_INPUT_FUNC_NAME));
        router_swap_args.push(managed_buffer!(WEGLD_TOKEN_ID));
        let half = BigUint::from(HALF_UNIT);
        router_swap_args.push(half.to_bytes_be_buffer());

        tasks.push((TaskType::RouterSwap, router_swap_args).into());

        // Send
        let mut send_args = ManagedVec::new();
        send_args.push(managed_buffer!(second_address.as_address().as_bytes()));
        tasks.push((TaskType::SendEgldOrEsdt, send_args).into());

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .typed(composable_tasks_proxy::ComposableTasksContractProxy)
            .compose_tasks(min_expected_token_out, tasks)
            .gas(100_000_000)
            .payment(EgldOrEsdtTokenPayment::new(
                EgldOrEsdtTokenIdentifier::egld(),
                token_nonce,
                BigUint::from(ONE_UNIT),
            ))
            .returns(ReturnsResult)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }
    async fn router_multi_pair_swaps(&mut self) {
        let second_address = &self.config.random_address;
        let egld_mex_pair_address = &self.config.egld_mex_pair_address;
        let egld_usdc_pair_address = &self.config.egld_usdc_pair_address;


        let min_expected_token_out = EgldOrEsdtTokenPayment::new(
            EgldOrEsdtTokenIdentifier::esdt(USDC_TOKEN_ID),
            0u64,
            BigUint::from(1u64),
        );

        let mut tasks = MultiValueEncoded::new();

        // Wrap EGLD
        // let no_args = ManagedVec::new();
        // tasks.push((TaskType::WrapEGLD, no_args).into());

        // // Swap fixed output
        // let mut swap_args_fixed_output = ManagedVec::new();
        // swap_args_fixed_output.push(managed_buffer!(SWAP_TOKENS_FIXED_OUTPUT_FUNC_NAME));
        // swap_args_fixed_output.push(managed_buffer!(USDC_TOKEN_ID));
        // let amount_38 = BigUint::from(30000000u64); // 38 units with 6 decimals
        // swap_args_fixed_output.push(amount_38.to_bytes_be_buffer()); 
        // tasks.push((TaskType::Swap, swap_args_fixed_output).into());


        // // Swap fixed input
        // let mut swap_args = ManagedVec::new();
        // swap_args.push(managed_buffer!(SWAP_TOKENS_FIXED_INPUT_FUNC_NAME));
        // swap_args.push(managed_buffer!(MEX_TOKEN_ID));
        // swap_args.push(managed_buffer!(b"1"));
        // tasks.push((TaskType::Swap, swap_args).into());

        // Router swap
        let mut router_swap_args = ManagedVec::new();
        router_swap_args.push(managed_buffer!(egld_mex_pair_address.as_address().as_bytes()));
        router_swap_args.push(managed_buffer!(SWAP_TOKENS_FIXED_INPUT_FUNC_NAME));
        router_swap_args.push(managed_buffer!(WEGLD_TOKEN_ID));
        router_swap_args.push(managed_buffer!(b"1"));

        router_swap_args.push(managed_buffer!(egld_usdc_pair_address.as_address().as_bytes()));
        router_swap_args.push(managed_buffer!(SWAP_TOKENS_FIXED_INPUT_FUNC_NAME));
        router_swap_args.push(managed_buffer!(USDC_TOKEN_ID));
        router_swap_args.push(managed_buffer!(b"1"));

        tasks.push((TaskType::RouterSwap, router_swap_args).into());

        // Send
        let mut send_args = ManagedVec::new();
        send_args.push(managed_buffer!(second_address.as_address().as_bytes()));
        tasks.push((TaskType::SendEgldOrEsdt, send_args).into());

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .typed(composable_tasks_proxy::ComposableTasksContractProxy)
            .compose_tasks(min_expected_token_out, tasks)
            .gas(100_000_000)
            .payment(EgldOrEsdtTokenPayment::new(
                EgldOrEsdtTokenIdentifier::esdt(MEX_TOKEN_ID),
                0u64,
                BigUint::from(ONE_UNIT * 3u64) * MILLION,
            ))
            .returns(ReturnsResult)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }
}
