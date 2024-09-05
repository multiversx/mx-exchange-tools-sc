#![allow(non_snake_case)]

mod auto_pos_creator_config;

use auto_pos_creator::auto_pos_creator_proxy;
use auto_pos_creator::external_sc_interactions::router_actions::SwapOperationType;
use auto_pos_creator_config::Config;
use multiversx_sc_snippets::imports::*;
use multiversx_sc_snippets::sdk;
use serde::{Deserialize, Serialize};
use std::{
    io::{Read, Write},
    path::Path,
};

const GATEWAY: &str = sdk::blockchain::DEVNET_GATEWAY;
const STATE_FILE: &str = "state.toml";
pub const SWAP_TOKENS_FIXED_INPUT_FUNC_NAME: &[u8] = b"swapTokensFixedInput";
pub const SWAP_TOKENS_FIXED_OUTPUT_FUNC_NAME: &[u8] = b"swapTokensFixedOutput";
pub static WEGLD_TOKEN_ID: &[u8] = b"WEGLD-a28c59";
pub static MEX_TOKEN_ID: &[u8] = b"MEX-a659d0";
pub static EGLDMEX_LP_TOKEN_ID: &[u8] = b"EGLDMEX-95c6d5";
pub static EGLDMEX_FARM_TOKEN_ID: &[u8] = b"EGLDMEXFL-f0bc2e";
pub static EGLDUTK_LP_TOKEN_ID: &[u8] = b"UTKWEGLD-4d60d6";
pub static EGLDUTK_FARM_TOKEN_ID: &[u8] = b"UTKWEGLDFL-082dbc";
pub static EGLDUTK_DUAL_YIELD_TOKEN_ID: &[u8] = b"METAUTKLK-6003e8";

pub static ONE_TOKEN_ID: &[u8] = b"ONE-83a7c0";
pub static USDC_TOKEN_ID: &[u8] = b"USDC-350c4e";
pub static UTK_TOKEN_ID: &[u8] = b"UTK-14d57d";
const HALF_UNIT: u64 = 500000000000000000; // 0.5
const ONE_UNIT: u64 = 1000000000000000000; // 0.5
const MILLION: u64 = 1_000_000;

#[tokio::main]
async fn main() {
    env_logger::init();

    let mut args = std::env::args();
    let _ = args.next();
    let cmd = args.next().expect("at least one argument required");
    let mut interact = ContractInteract::new().await;
    match cmd.as_str() {
        "deploy" => interact.deploy().await,
        "createLpPosFromSingleToken" => interact.create_lp_pos_from_single_token().await,
        "createFarmPosFromSingleToken" => interact.create_farm_pos_from_single_token().await,
        "createFarmPosFromTwoTokens" => interact.create_farm_pos_from_two_tokens().await,
        "createMetastakingPosFromSingleToken" => {
            interact.create_metastaking_pos_from_single_token().await
        }
        "createMetastakingPosFromLpToken" => interact.create_metastaking_pos_from_lp_token().await,
        "createMetastakingPosFromTwoTokens" => {
            interact.create_metastaking_pos_from_two_tokens().await
        }
        "createMetastakingWithMergeThroughPosCreator" => {
            interact
                .create_metastaking_with_merge_through_pos_creator_test()
                .await
        }

        "createFarmStakingPosFromSingleToken" => {
            interact.create_farm_staking_pos_from_single_token().await
        }
        "exitMetastakingPos" => interact.exit_metastaking_pos_endpoint().await,
        "exitFarmPos" => interact.exit_farm_pos().await,
        "exitLpPos" => interact.exit_lp_pos().await,
        "tryExitWrongAddressTest" => interact.try_exit_wrong_address_test().await,
        "tryCreateLpImpossibleSwapPath" => interact.try_create_lp_impossible_swap_path().await,
        "tryCreateLpFromTwoWrongTokens" => interact.try_create_lp_from_wrong_tokens().await,
        "tryCreateLpPosFromSameLpToken" => {
            interact.try_create_lp_pos_from_same_lp_token_test().await
        }
        "tryCreateLpPosFromDifferentLpToken" => {
            interact
                .try_create_lp_pos_from_different_lp_token_test()
                .await
        }
        "tryCreateLpPosFromFarmPos" => interact.try_create_lp_pos_from_farm_pos_test().await,
        "tryCreatePositionFromTwoWrongTokens" => {
            interact.try_create_pos_from_two_wrong_tokens_test().await
        }
        "tryCreatePositionWrongSlippage" => interact.try_create_pos_wrong_slippage_test().await,
        "tryCreatePositionMultiplePathsLastOneFailling" => {
            interact
                .try_create_pos_multiple_paths_last_one_failling_test()
                .await
        }
        "tryCreatePositionWrongFarmAddress" => {
            interact.try_create_pos_wrong_farm_address_test().await
        }

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
            "mxsc:../output/auto-pos-creator.mxsc.json",
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
        let egld_wrapper_address = &self.config.wegld_address;
        let router_address = &self.config.router_address;

        let new_address = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .typed(auto_pos_creator_proxy::AutoPosCreatorProxy)
            .init(egld_wrapper_address, router_address)
            .code(&self.contract_code)
            .gas(60_000_000)
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

    async fn create_lp_pos_from_single_token(&mut self) {
        let egld_utk_pair_address = &self.config.egld_utk_pair_address;
        let egld_one_pair_address = &self.config.egld_one_pair_address;

        let add_liq_first_token_min_amount_out = BigUint::<StaticApi>::from(1u128);
        let add_liq_second_token_min_amount_out = BigUint::<StaticApi>::from(1u128);
        let mut swap_operations = MultiValueEncoded::new();
        let swap_operation: SwapOperationType<StaticApi> = (
            managed_address!(egld_one_pair_address.as_address()),
            ManagedBuffer::from(SWAP_TOKENS_FIXED_INPUT_FUNC_NAME),
            managed_token_id!(WEGLD_TOKEN_ID), // Want token
            BigUint::from(1u64),
        )
            .into();
        swap_operations.push(swap_operation);

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .typed(auto_pos_creator_proxy::AutoPosCreatorProxy)
            .create_lp_pos_from_single_token(
                egld_utk_pair_address,
                add_liq_first_token_min_amount_out,
                add_liq_second_token_min_amount_out,
                swap_operations,
            )
            .payment(EgldOrEsdtTokenPayment::new(
                EgldOrEsdtTokenIdentifier::esdt(ONE_TOKEN_ID),
                0u64,
                BigUint::from(ONE_UNIT) * 10u64,
            ))
            .gas(100_000_000)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn create_farm_pos_from_single_token(&mut self) {
        let egld_utk_farm_address = &self.config.egld_utk_farm_address;
        let egld_one_pair_address = &self.config.egld_one_pair_address;

        let add_liq_first_token_min_amount_out = BigUint::<StaticApi>::from(1u128);
        let add_liq_second_token_min_amount_out = BigUint::<StaticApi>::from(1u128);
        let mut swap_operations = MultiValueEncoded::new();
        let swap_operation: SwapOperationType<StaticApi> = (
            managed_address!(egld_one_pair_address.as_address()),
            ManagedBuffer::from(SWAP_TOKENS_FIXED_INPUT_FUNC_NAME),
            managed_token_id!(WEGLD_TOKEN_ID), // Want token
            BigUint::from(1u64),
        )
            .into();
        swap_operations.push(swap_operation);

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .typed(auto_pos_creator_proxy::AutoPosCreatorProxy)
            .create_farm_pos_from_single_token(
                egld_utk_farm_address,
                add_liq_first_token_min_amount_out,
                add_liq_second_token_min_amount_out,
                swap_operations,
            )
            .payment(EgldOrEsdtTokenPayment::new(
                EgldOrEsdtTokenIdentifier::esdt(ONE_TOKEN_ID),
                0u64,
                BigUint::from(ONE_UNIT) * 100u64,
            ))
            .gas(100_000_000)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Returned payments: {response:?}");
    }

    async fn create_farm_pos_from_two_tokens(&mut self) {
        let farm_address = bech32::decode("");
        let add_liq_first_token_min_amount_out = BigUint::<StaticApi>::from(1u128);
        let add_liq_second_token_min_amount_out = BigUint::<StaticApi>::from(1u128);

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .typed(auto_pos_creator_proxy::AutoPosCreatorProxy)
            .create_farm_pos_from_two_tokens(
                farm_address,
                add_liq_first_token_min_amount_out,
                add_liq_second_token_min_amount_out,
            )
            .payment(EgldOrEsdtTokenPayment::new(
                EgldOrEsdtTokenIdentifier::esdt(ONE_TOKEN_ID),
                0u64,
                BigUint::from(ONE_UNIT) * 100u64,
            ))
            .gas(100_000_000)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn create_metastaking_pos_from_single_token(&mut self) {
        let egld_one_pair_address = &self.config.egld_one_pair_address;

        let metastaking_address = &self.config.metastaking_utk_address;
        let add_liq_first_token_min_amount_out = BigUint::<StaticApi>::from(1u128);
        let add_liq_second_token_min_amount_out = BigUint::<StaticApi>::from(1u128);
        let mut swap_operations = MultiValueEncoded::new();
        let swap_operation: SwapOperationType<StaticApi> = (
            managed_address!(&egld_one_pair_address.as_address()),
            ManagedBuffer::from(SWAP_TOKENS_FIXED_INPUT_FUNC_NAME),
            managed_token_id!(WEGLD_TOKEN_ID), // Want token A
            BigUint::from(1u64),
        )
            .into();
        swap_operations.push(swap_operation);

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .typed(auto_pos_creator_proxy::AutoPosCreatorProxy)
            .create_metastaking_pos_from_single_token(
                metastaking_address,
                add_liq_first_token_min_amount_out,
                add_liq_second_token_min_amount_out,
                swap_operations,
            )
            .payment(EgldOrEsdtTokenPayment::new(
                EgldOrEsdtTokenIdentifier::esdt(ONE_TOKEN_ID),
                0u64,
                BigUint::from(ONE_UNIT) * 100u64,
            ))
            .gas(100_000_000)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn create_metastaking_pos_from_lp_token(&mut self) {
        let metastaking_address = &self.config.metastaking_utk_address;
        let add_liq_first_token_min_amount_out = BigUint::<StaticApi>::from(1u128);
        let add_liq_second_token_min_amount_out = BigUint::<StaticApi>::from(1u128);

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .typed(auto_pos_creator_proxy::AutoPosCreatorProxy)
            .create_metastaking_pos_from_single_token(
                metastaking_address,
                add_liq_first_token_min_amount_out,
                add_liq_second_token_min_amount_out,
                MultiValueEncoded::new(),
            )
            .payment(EgldOrEsdtTokenPayment::new(
                EgldOrEsdtTokenIdentifier::esdt(EGLDUTK_FARM_TOKEN_ID),
                0u64,
                BigUint::from(10365193609537230u64),
            ))
            .gas(100_000_000)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn create_metastaking_pos_from_two_tokens(&mut self) {
        let metastaking_address = &self.config.metastaking_utk_address;
        let add_liq_first_token_min_amount_out = BigUint::<StaticApi>::from(1u128);
        let add_liq_second_token_min_amount_out = BigUint::<StaticApi>::from(1u128);

        let mut multi_payments = MultiEsdtPayment::new();
        multi_payments.push(EsdtTokenPayment::new(
            TokenIdentifier::from(UTK_TOKEN_ID),
            0u64,
            BigUint::from(ONE_UNIT),
        ));
        multi_payments.push(EsdtTokenPayment::new(
            TokenIdentifier::from(WEGLD_TOKEN_ID),
            0u64,
            BigUint::from(HALF_UNIT),
        ));

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .typed(auto_pos_creator_proxy::AutoPosCreatorProxy)
            .create_metastaking_pos_from_two_tokens(
                metastaking_address,
                add_liq_first_token_min_amount_out,
                add_liq_second_token_min_amount_out,
            )
            .payment(multi_payments)
            .gas(100_000_000)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn create_metastaking_with_merge_through_pos_creator_test(&mut self) {
        let metastaking_address = &self.config.metastaking_utk_address;
        let add_liq_first_token_min_amount_out = BigUint::<StaticApi>::from(1u128);
        let add_liq_second_token_min_amount_out = BigUint::<StaticApi>::from(1u128);

        let mut multi_payments = MultiEsdtPayment::new();
        multi_payments.push(EsdtTokenPayment::new(
            TokenIdentifier::from(EGLDUTK_LP_TOKEN_ID),
            0u64,
            BigUint::from(17158680783319457u64),
        ));
        multi_payments.push(EsdtTokenPayment::new(
            TokenIdentifier::from(EGLDUTK_DUAL_YIELD_TOKEN_ID),
            6u64,
            BigUint::from(ONE_UNIT),
        ));

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .typed(auto_pos_creator_proxy::AutoPosCreatorProxy)
            .create_metastaking_pos_from_single_token(
                metastaking_address,
                add_liq_first_token_min_amount_out,
                add_liq_second_token_min_amount_out,
                MultiValueEncoded::new(),
            )
            .payment(multi_payments)
            .gas(100_000_000)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn create_farm_staking_pos_from_single_token(&mut self) {
        let farm_staking_address = &self.config.farm_staking_utk_address;
        let egld_utk_pair_address = &self.config.egld_utk_pair_address;

        let min_amount_out = BigUint::<StaticApi>::from(100u128);
        let mut swap_operations = MultiValueEncoded::new();
        let swap_operation: SwapOperationType<StaticApi> = (
            managed_address!(egld_utk_pair_address.as_address()),
            ManagedBuffer::from(SWAP_TOKENS_FIXED_INPUT_FUNC_NAME),
            managed_token_id!(UTK_TOKEN_ID), // Want token B
            BigUint::from(1u64),
        )
            .into();
        swap_operations.push(swap_operation);

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .typed(auto_pos_creator_proxy::AutoPosCreatorProxy)
            .create_farm_staking_pos_from_single_token(
                farm_staking_address,
                min_amount_out,
                swap_operations,
            )
            .payment(EgldOrEsdtTokenPayment::new(
                EgldOrEsdtTokenIdentifier::esdt(WEGLD_TOKEN_ID),
                0u64,
                BigUint::from(HALF_UNIT) / 2u64,
            ))
            .gas(100_000_000)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn exit_metastaking_pos_endpoint(&mut self) {
        let metastaking_address = &self.config.metastaking_utk_address;
        let first_token_min_amount_out = BigUint::<StaticApi>::from(1u128);
        let second_token_min_amont_out = BigUint::<StaticApi>::from(1u128);

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .typed(auto_pos_creator_proxy::AutoPosCreatorProxy)
            .exit_metastaking_pos_endpoint(
                metastaking_address,
                first_token_min_amount_out,
                second_token_min_amont_out,
            )
            .payment((
                TokenIdentifier::from(EGLDUTK_DUAL_YIELD_TOKEN_ID),
                4u64,
                BigUint::from(5536184322936854847u64),
            ))
            .gas(100_000_000)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn exit_farm_pos(&mut self) {
        let egld_utk_farm_address = &self.config.egld_utk_farm_address;

        let first_token_min_amount_out = BigUint::<StaticApi>::from(1u128);
        let second_token_min_amont_out = BigUint::<StaticApi>::from(1u128);

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .typed(auto_pos_creator_proxy::AutoPosCreatorProxy)
            .exit_farm_pos(
                egld_utk_farm_address,
                first_token_min_amount_out,
                second_token_min_amont_out,
            )
            .payment((
                TokenIdentifier::from(EGLDUTK_FARM_TOKEN_ID),
                8u64,
                BigUint::from(103679037549679789u64),
            ))
            .gas(100_000_000)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn exit_lp_pos(&mut self) {
        let egld_mex_pair_address = &self.config.egld_mex_pair_address;
        let first_token_min_amount_out = BigUint::<StaticApi>::from(1u128);
        let second_token_min_amont_out = BigUint::<StaticApi>::from(1u128);

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .typed(auto_pos_creator_proxy::AutoPosCreatorProxy)
            .exit_lp_pos(
                egld_mex_pair_address,
                first_token_min_amount_out,
                second_token_min_amont_out,
            )
            .payment((
                TokenIdentifier::from(EGLDMEX_LP_TOKEN_ID),
                0u64,
                BigUint::from(276493421633915622u128),
            ))
            .gas(50_000_000)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    //// NEGATIVE Tests

    async fn try_create_lp_impossible_swap_path(&mut self) {
        let egld_mex_pair_address = &self.config.egld_mex_pair_address;
        let egld_usdc_pair_address = &self.config.egld_usdc_pair_address;

        let add_liq_first_token_min_amount_out = BigUint::<StaticApi>::from(1u128);
        let add_liq_second_token_min_amount_out = BigUint::<StaticApi>::from(1u128);
        let mut swap_operations = MultiValueEncoded::new();
        let swap_operation: SwapOperationType<StaticApi> = (
            managed_address!(egld_usdc_pair_address.as_address()),
            ManagedBuffer::from(SWAP_TOKENS_FIXED_INPUT_FUNC_NAME),
            managed_token_id!(MEX_TOKEN_ID), // Want token
            BigUint::from(1u64),
        )
            .into();
        swap_operations.push(swap_operation);

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .typed(auto_pos_creator_proxy::AutoPosCreatorProxy)
            .create_lp_pos_from_single_token(
                egld_mex_pair_address,
                add_liq_first_token_min_amount_out,
                add_liq_second_token_min_amount_out,
                swap_operations,
            )
            .payment(EgldOrEsdtTokenPayment::new(
                EgldOrEsdtTokenIdentifier::esdt(WEGLD_TOKEN_ID),
                0u64,
                BigUint::from(HALF_UNIT),
            ))
            .gas(50_000_000)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn try_create_lp_from_wrong_tokens(&mut self) {
        let egld_mex_pair_address = &self.config.egld_mex_pair_address;
        let add_liq_first_token_min_amount_out = BigUint::<StaticApi>::from(1u128);
        let add_liq_second_token_min_amount_out = BigUint::<StaticApi>::from(1u128);

        let mut multi_payments = MultiEsdtPayment::new();
        multi_payments.push(EsdtTokenPayment::new(
            TokenIdentifier::from(MEX_TOKEN_ID),
            0u64,
            BigUint::from(3 * ONE_UNIT) * BigUint::from(MILLION),
        ));
        multi_payments.push(EsdtTokenPayment::new(
            TokenIdentifier::from(USDC_TOKEN_ID),
            0u64,
            BigUint::from(1_000_000u64),
        ));

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .typed(auto_pos_creator_proxy::AutoPosCreatorProxy)
            .create_lp_pos_from_two_tokens(
                egld_mex_pair_address,
                add_liq_first_token_min_amount_out,
                add_liq_second_token_min_amount_out,
            )
            .payment(multi_payments)
            .returns(ReturnsResultUnmanaged)
            .gas(50_000_000)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn try_exit_wrong_address_test(&mut self) {
        let egld_usdc_pair_address = &self.config.egld_usdc_pair_address;
        let egld_utk_farm_address = &self.config.egld_utk_farm_address;
        let metastaking_utk_address = &self.config.metastaking_utk_address;

        // LP
        let response_lp = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .typed(auto_pos_creator_proxy::AutoPosCreatorProxy)
            .exit_lp_pos(
                egld_usdc_pair_address,
                BigUint::from(1u64),
                BigUint::from(1u64),
            )
            .payment((
                TokenIdentifier::from(EGLDMEX_LP_TOKEN_ID),
                0u64,
                BigUint::from(276493421633915622u128),
            ))
            .gas(50_000_000)
            .returns(ExpectError(4, "error signalled by smartcontract"))
            .prepare_async()
            .run()
            .await;

        println!("Result: {response_lp:?}");

        // FARM
        let response_farm = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .typed(auto_pos_creator_proxy::AutoPosCreatorProxy)
            .exit_farm_pos(
                egld_utk_farm_address,
                BigUint::from(1u64),
                BigUint::from(1u64),
            )
            .payment((
                TokenIdentifier::from(EGLDMEX_LP_TOKEN_ID),
                0u64,
                BigUint::from(276493421633915622u128),
            ))
            .gas(50_000_000)
            .returns(ExpectError(4, "error signalled by smartcontract"))
            .prepare_async()
            .run()
            .await;

        println!("Result: {response_farm:?}");

        // Metastaking
        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .typed(auto_pos_creator_proxy::AutoPosCreatorProxy)
            .exit_metastaking_pos_endpoint(
                metastaking_utk_address,
                BigUint::from(1u64),
                BigUint::from(1u64),
            )
            .payment((
                TokenIdentifier::from(EGLDMEX_LP_TOKEN_ID),
                0u64,
                BigUint::from(276493421633915622u128),
            ))
            .gas(50_000_000)
            .returns(ExpectError(4, "error signalled by smartcontract"))
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn try_create_lp_pos_from_same_lp_token_test(&mut self) {
        let egld_utk_pair_address = &self.config.egld_utk_pair_address;

        let add_liq_first_token_min_amount_out = BigUint::<StaticApi>::from(1u128);
        let add_liq_second_token_min_amount_out = BigUint::<StaticApi>::from(1u128);

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .typed(auto_pos_creator_proxy::AutoPosCreatorProxy)
            .create_lp_pos_from_single_token(
                egld_utk_pair_address,
                add_liq_first_token_min_amount_out,
                add_liq_second_token_min_amount_out,
                MultiValueEncoded::new(),
            )
            .payment(EgldOrEsdtTokenPayment::new(
                EgldOrEsdtTokenIdentifier::esdt(EGLDUTK_LP_TOKEN_ID),
                0u64,
                BigUint::from(64839320423353075u64),
            ))
            .gas(100_000_000)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn try_create_lp_pos_from_different_lp_token_test(&mut self) {
        let egld_utk_pair_address = &self.config.egld_utk_pair_address;

        let add_liq_first_token_min_amount_out = BigUint::<StaticApi>::from(1u128);
        let add_liq_second_token_min_amount_out = BigUint::<StaticApi>::from(1u128);

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .typed(auto_pos_creator_proxy::AutoPosCreatorProxy)
            .create_lp_pos_from_single_token(
                egld_utk_pair_address,
                add_liq_first_token_min_amount_out,
                add_liq_second_token_min_amount_out,
                MultiValueEncoded::new(),
            )
            .payment(EgldOrEsdtTokenPayment::new(
                EgldOrEsdtTokenIdentifier::esdt(EGLDMEX_LP_TOKEN_ID),
                0u64,
                BigUint::from(ONE_UNIT),
            ))
            .gas(100_000_000)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn try_create_lp_pos_from_farm_pos_test(&mut self) {
        let egld_utk_pair_address = &self.config.egld_utk_pair_address;

        let add_liq_first_token_min_amount_out = BigUint::<StaticApi>::from(1u128);
        let add_liq_second_token_min_amount_out = BigUint::<StaticApi>::from(1u128);

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .typed(auto_pos_creator_proxy::AutoPosCreatorProxy)
            .create_lp_pos_from_single_token(
                egld_utk_pair_address,
                add_liq_first_token_min_amount_out,
                add_liq_second_token_min_amount_out,
                MultiValueEncoded::new(),
            )
            .payment(EgldOrEsdtTokenPayment::new(
                EgldOrEsdtTokenIdentifier::esdt(EGLDUTK_FARM_TOKEN_ID),
                9u64,
                BigUint::from(ONE_UNIT) / 10u64,
            ))
            .gas(100_000_000)
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn try_create_pos_from_two_wrong_tokens_test(&mut self) {
        let egld_utk_farm_address = &self.config.egld_utk_farm_address;
        let egld_utk_pair_address = &self.config.egld_utk_pair_address;
        let metastaking_utk_address = &self.config.metastaking_utk_address;
        let farm_staking_utk_address = &self.config.farm_staking_utk_address;

        let add_liq_first_token_min_amount_out = BigUint::<StaticApi>::from(1u128);
        let add_liq_second_token_min_amount_out = BigUint::<StaticApi>::from(1u128);

        let mut multi_payments = MultiEsdtPayment::new();
        multi_payments.push(EsdtTokenPayment::new(
            TokenIdentifier::from(MEX_TOKEN_ID),
            0u64,
            BigUint::from(3 * ONE_UNIT) * BigUint::from(MILLION),
        ));
        multi_payments.push(EsdtTokenPayment::new(
            TokenIdentifier::from(USDC_TOKEN_ID),
            0u64,
            BigUint::from(1_000_000u64),
        ));

        // FARM
        let farm_response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .typed(auto_pos_creator_proxy::AutoPosCreatorProxy)
            .create_farm_pos_from_two_tokens(
                egld_utk_farm_address,
                BigUint::from(1u64),
                BigUint::from(1u64),
            )
            .payment(multi_payments.clone())
            .gas(100_000_000)
            .returns(ExpectError(4, "error signalled by smartcontract"))
            .prepare_async()
            .run()
            .await;

        println!("Result: {farm_response:?}");

        // LP
        let lp_response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .typed(auto_pos_creator_proxy::AutoPosCreatorProxy)
            .create_lp_pos_from_two_tokens(
                egld_utk_pair_address,
                add_liq_first_token_min_amount_out,
                add_liq_second_token_min_amount_out,
            )
            .payment(multi_payments.clone())
            .gas(100_000_000)
            .returns(ExpectError(4, "error signalled by smartcontract"))
            .prepare_async()
            .run()
            .await;

        println!("Result: {lp_response:?}");

        // Metastaking
        let metastaking_response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .typed(auto_pos_creator_proxy::AutoPosCreatorProxy)
            .create_metastaking_pos_from_two_tokens(
                metastaking_utk_address,
                BigUint::from(1u64),
                BigUint::from(1u64),
            )
            .payment(multi_payments)
            .gas(100_000_000)
            .returns(ExpectError(4, "error signalled by smartcontract"))
            .prepare_async()
            .run()
            .await;

        println!("Result: {metastaking_response:?}");

        // Farm staking
        let farm_staking_response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .typed(auto_pos_creator_proxy::AutoPosCreatorProxy)
            .create_farm_staking_pos_from_single_token(
                farm_staking_utk_address,
                BigUint::from(100u64),
                MultiValueEncoded::new(),
            )
            .payment(EgldOrEsdtTokenPayment::new(
                EgldOrEsdtTokenIdentifier::esdt(ONE_TOKEN_ID),
                0u64,
                BigUint::from(ONE_UNIT) * 100u64,
            ))
            .gas(100_000_000)
            .returns(ExpectError(4, "error signalled by smartcontract"))
            .prepare_async()
            .run()
            .await;

        println!("Result: {farm_staking_response:?}");
    }

    async fn try_create_pos_wrong_slippage_test(&mut self) {
        let egld_utk_farm_address = &self.config.egld_utk_farm_address;
        let egld_utk_pair_address = &self.config.egld_utk_pair_address;
        let egld_one_pair_address = &self.config.egld_one_pair_address;
        let metastaking_utk_address = &self.config.metastaking_utk_address;
        let farm_staking_utk_address = &self.config.farm_staking_utk_address;

        let mut swap_operations = MultiValueEncoded::new();
        let swap_operation: SwapOperationType<StaticApi> = (
            managed_address!(egld_one_pair_address.as_address()),
            ManagedBuffer::from(SWAP_TOKENS_FIXED_INPUT_FUNC_NAME),
            managed_token_id!(WEGLD_TOKEN_ID), // Want token
            BigUint::from(ONE_UNIT),
        )
            .into();
        swap_operations.push(swap_operation);

        // LP
        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .typed(auto_pos_creator_proxy::AutoPosCreatorProxy)
            .create_lp_pos_from_single_token(
                egld_utk_pair_address,
                BigUint::from(1u64),
                BigUint::from(1u64),
                swap_operations.clone(),
            )
            .payment(EgldOrEsdtTokenPayment::new(
                EgldOrEsdtTokenIdentifier::esdt(ONE_TOKEN_ID),
                0u64,
                BigUint::from(ONE_UNIT) * 100u64,
            ))
            .gas(100_000_000)
            .returns(ExpectError(4, "execution failed"))
            .prepare_async()
            .run()
            .await;
        println!("Returned payments: {response:?}");

        // Farm
        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .typed(auto_pos_creator_proxy::AutoPosCreatorProxy)
            .create_farm_pos_from_single_token(
                egld_utk_farm_address,
                BigUint::from(1u64),
                BigUint::from(1u64),
                swap_operations.clone(),
            )
            .payment(EgldOrEsdtTokenPayment::new(
                EgldOrEsdtTokenIdentifier::esdt(ONE_TOKEN_ID),
                0u64,
                BigUint::from(ONE_UNIT) * 100u64,
            ))
            .gas(100_000_000)
            .returns(ExpectError(4, "execution failed"))
            .prepare_async()
            .run()
            .await;
        println!("Returned payments: {response:?}");

        // Metastaking
        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .typed(auto_pos_creator_proxy::AutoPosCreatorProxy)
            .create_metastaking_pos_from_single_token(
                metastaking_utk_address,
                BigUint::from(1u64),
                BigUint::from(1u64),
                swap_operations.clone(),
            )
            .payment(EgldOrEsdtTokenPayment::new(
                EgldOrEsdtTokenIdentifier::esdt(ONE_TOKEN_ID),
                0u64,
                BigUint::from(ONE_UNIT) * 100u64,
            ))
            .gas(100_000_000)
            .returns(ExpectError(4, "execution failed"))
            .prepare_async()
            .run()
            .await;
        println!("Returned payments: {response:?}");

        // Farm staking
        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .typed(auto_pos_creator_proxy::AutoPosCreatorProxy)
            .create_farm_staking_pos_from_single_token(
                farm_staking_utk_address,
                BigUint::from(100u64),
                swap_operations,
            )
            .payment(EgldOrEsdtTokenPayment::new(
                EgldOrEsdtTokenIdentifier::esdt(ONE_TOKEN_ID),
                0u64,
                BigUint::from(ONE_UNIT) * 100u64,
            ))
            .gas(100_000_000)
            .returns(ExpectError(4, "execution failed"))
            .prepare_async()
            .run()
            .await;
        println!("Returned payments: {response:?}");
    }

    async fn try_create_pos_multiple_paths_last_one_failling_test(&mut self) {
        let egld_utk_farm_address = &self.config.egld_utk_farm_address;
        let egld_one_pair_address = &self.config.egld_one_pair_address;
        let egld_usdc_pair_address = &self.config.egld_usdc_pair_address;
        let metastaking_utk_address = &self.config.metastaking_utk_address;
        let farm_staking_utk_address = &self.config.farm_staking_utk_address;

        let mut swap_operations = MultiValueEncoded::new();
        let swap_operation: SwapOperationType<StaticApi> = (
            managed_address!(egld_one_pair_address.as_address()),
            ManagedBuffer::from(SWAP_TOKENS_FIXED_INPUT_FUNC_NAME),
            managed_token_id!(WEGLD_TOKEN_ID), // Want token
            BigUint::from(1u64),
        )
            .into();
        swap_operations.push(swap_operation);
        let swap_operation: SwapOperationType<StaticApi> = (
            managed_address!(egld_usdc_pair_address.as_address()),
            ManagedBuffer::from(SWAP_TOKENS_FIXED_INPUT_FUNC_NAME),
            managed_token_id!(USDC_TOKEN_ID), // Want token
            BigUint::from(1u64),
        )
            .into();
        swap_operations.push(swap_operation);
        let swap_operation: SwapOperationType<StaticApi> = (
            managed_address!(egld_usdc_pair_address.as_address()),
            ManagedBuffer::from(SWAP_TOKENS_FIXED_INPUT_FUNC_NAME),
            managed_token_id!(WEGLD_TOKEN_ID), // Want token
            BigUint::from(ONE_UNIT),
        )
            .into();
        swap_operations.push(swap_operation);

        let lp_response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .typed(auto_pos_creator_proxy::AutoPosCreatorProxy)
            .create_lp_pos_from_single_token(
                egld_utk_farm_address,
                BigUint::from(1u64),
                BigUint::from(1u64),
                swap_operations.clone(),
            )
            .payment(EgldOrEsdtTokenPayment::new(
                EgldOrEsdtTokenIdentifier::esdt(ONE_TOKEN_ID),
                0u64,
                BigUint::from(ONE_UNIT) * 100u64,
            ))
            .gas(100_000_000)
            .returns(ExpectError(4, "execution failed"))
            .prepare_async()
            .run()
            .await;

        println!("Returned payments: {lp_response:?}");

        let farm_response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .typed(auto_pos_creator_proxy::AutoPosCreatorProxy)
            .create_farm_pos_from_single_token(
                egld_utk_farm_address,
                BigUint::from(1u64),
                BigUint::from(1u64),
                swap_operations.clone(),
            )
            .payment(EgldOrEsdtTokenPayment::new(
                EgldOrEsdtTokenIdentifier::esdt(ONE_TOKEN_ID),
                0u64,
                BigUint::from(ONE_UNIT) * 100u64,
            ))
            .gas(100_000_000)
            .returns(ExpectError(4, "execution failed"))
            .prepare_async()
            .run()
            .await;

        println!("Returned payments: {farm_response:?}");

        let metastaking_response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .typed(auto_pos_creator_proxy::AutoPosCreatorProxy)
            .create_metastaking_pos_from_single_token(
                metastaking_utk_address,
                BigUint::from(1u64),
                BigUint::from(1u64),
                swap_operations.clone(),
            )
            .payment(EgldOrEsdtTokenPayment::new(
                EgldOrEsdtTokenIdentifier::esdt(ONE_TOKEN_ID),
                0u64,
                BigUint::from(ONE_UNIT) * 100u64,
            ))
            .gas(100_000_000)
            .returns(ExpectError(4, "execution failed"))
            .prepare_async()
            .run()
            .await;

        println!("Returned payments: {metastaking_response:?}");

        let farm_staking_response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .typed(auto_pos_creator_proxy::AutoPosCreatorProxy)
            .create_farm_staking_pos_from_single_token(
                farm_staking_utk_address,
                BigUint::from(100u64),
                swap_operations,
            )
            .payment(EgldOrEsdtTokenPayment::new(
                EgldOrEsdtTokenIdentifier::esdt(ONE_TOKEN_ID),
                0u64,
                BigUint::from(ONE_UNIT) * 100u64,
            ))
            .gas(100_000_000)
            .returns(ExpectError(4, "execution failed"))
            .prepare_async()
            .run()
            .await;

        println!("Returned payments: {farm_staking_response:?}");
    }

    async fn try_create_pos_wrong_farm_address_test(&mut self) {
        let egld_utk_farm_address = &self.config.egld_utk_farm_address;
        let egld_one_pair_address = &self.config.egld_one_pair_address;
        let metastaking_utk_address = &self.config.metastaking_utk_address;
        let farm_staking_utk_address = &self.config.farm_staking_utk_address;

        let mut swap_operations = MultiValueEncoded::new();
        let swap_operation: SwapOperationType<StaticApi> = (
            managed_address!(egld_one_pair_address.as_address()),
            ManagedBuffer::from(SWAP_TOKENS_FIXED_INPUT_FUNC_NAME),
            managed_token_id!(WEGLD_TOKEN_ID), // Want token
            BigUint::from(1u64),
        )
            .into();
        swap_operations.push(swap_operation);

        // LP
        let farm_response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .typed(auto_pos_creator_proxy::AutoPosCreatorProxy)
            .create_lp_pos_from_single_token(
                egld_utk_farm_address, //this should be a LP address
                BigUint::from(1u64),
                BigUint::from(1u64),
                swap_operations.clone(),
            )
            .payment(EgldOrEsdtTokenPayment::new(
                EgldOrEsdtTokenIdentifier::esdt(ONE_TOKEN_ID),
                0u64,
                BigUint::from(ONE_UNIT) * 100u64,
            ))
            .gas(100_000_000)
            .returns(ExpectError(4, "Invalid token ID"))
            .prepare_async()
            .run()
            .await;

        println!("Returned payments: {farm_response:?}");

        // Farm
        let farm_response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .typed(auto_pos_creator_proxy::AutoPosCreatorProxy)
            .create_farm_pos_from_single_token(
                egld_one_pair_address, //this should be a farm address
                BigUint::from(1u64),
                BigUint::from(1u64),
                swap_operations.clone(),
            )
            .payment(EgldOrEsdtTokenPayment::new(
                EgldOrEsdtTokenIdentifier::esdt(ONE_TOKEN_ID),
                0u64,
                BigUint::from(ONE_UNIT) * 100u64,
            ))
            .gas(100_000_000)
            .returns(ExpectError(4, "storage decode error: bad array length"))
            .prepare_async()
            .run()
            .await;

        println!("Returned payments: {farm_response:?}");

        // Metastaking
        let farm_response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .typed(auto_pos_creator_proxy::AutoPosCreatorProxy)
            .create_metastaking_pos_from_single_token(
                farm_staking_utk_address, //this should be a metastaking address
                BigUint::from(1u64),
                BigUint::from(1u64),
                swap_operations.clone(),
            )
            .payment(EgldOrEsdtTokenPayment::new(
                EgldOrEsdtTokenIdentifier::esdt(ONE_TOKEN_ID),
                0u64,
                BigUint::from(ONE_UNIT) * 100u64,
            ))
            .gas(100_000_000)
            .returns(ExpectError(4, "storage decode error: bad array length"))
            .prepare_async()
            .run()
            .await;

        println!("Returned payments: {farm_response:?}");

        // Farm staking
        let farm_response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .typed(auto_pos_creator_proxy::AutoPosCreatorProxy)
            .create_farm_staking_pos_from_single_token(
                metastaking_utk_address, //this should be a farm staking address
                BigUint::from(100u64),
                swap_operations,
            )
            .payment(EgldOrEsdtTokenPayment::new(
                EgldOrEsdtTokenIdentifier::esdt(ONE_TOKEN_ID),
                0u64,
                BigUint::from(ONE_UNIT) * 100u64,
            ))
            .gas(100_000_000)
            .returns(ExpectError(4, "Invalid swap output token identifier"))
            .prepare_async()
            .run()
            .await;

        println!("Returned payments: {farm_response:?}");
    }
}
