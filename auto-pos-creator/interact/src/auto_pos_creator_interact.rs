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
pub static EGLDUTK_FARM_TOKEN_ID: &[u8] = b"UTKWEGLDFL-478337";

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
        "createMetastakingPosFromTwoTokens" => {
            interact.create_metastaking_pos_from_two_tokens().await
        }
        "createFarmStakingPosFromSingleToken" => {
            interact.create_farm_staking_pos_from_single_token().await
        }
        "exitMetastakingPos" => interact.exit_metastaking_pos_endpoint().await,
        "exitFarmPos" => interact.exit_farm_pos().await,
        "exitLpPos" => interact.exit_lp_pos().await,
        "tryExitLpWrongAddressTest" => interact.try_exit_lp_wrong_address_test().await,
        "tryCreateLpImpossibleSwapPath" => interact.try_create_lp_impossible_swap_path().await,
        "tryCreateLpFromTwoWrongTokens" => interact.try_create_lp_from_wrong_tokens().await,
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
        let egld_mex_pair_address = &self.config.egld_mex_pair_address;

        let add_liq_first_token_min_amount_out = BigUint::<StaticApi>::from(1u128);
        let add_liq_second_token_min_amount_out = BigUint::<StaticApi>::from(1u128);
        let mut swap_operations = MultiValueEncoded::new();
        let swap_operation: SwapOperationType<StaticApi> = (
            managed_address!(egld_mex_pair_address.as_address()),
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
        let token_id = String::new();
        let token_nonce = 0u64;
        let token_amount = BigUint::<StaticApi>::from(0u128);

        let farm_address = bech32::decode("");
        let add_liq_first_token_min_amount_out = BigUint::<StaticApi>::from(0u128);
        let add_liq_second_token_min_amount_out = BigUint::<StaticApi>::from(0u128);

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
            .payment((
                TokenIdentifier::from(token_id.as_str()),
                token_nonce,
                token_amount,
            ))
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn create_metastaking_pos_from_single_token(&mut self) {
        let token_id = String::new();
        let token_nonce = 0u64;
        let token_amount = BigUint::<StaticApi>::from(0u128);

        let metastaking_address = bech32::decode("");
        let add_liq_first_token_min_amount_out = BigUint::<StaticApi>::from(0u128);
        let add_liq_second_token_min_amount_out = BigUint::<StaticApi>::from(0u128);
        // let swap_operations = MultiValueVec::from(vec![MultiValue4::from((bech32::decode(""), ManagedBuffer::new_from_bytes(&b""[..]), TokenIdentifier::from_esdt_bytes(&b""[..]), BigUint::<StaticApi>::from(0u128)))]);
        let swap_operations = MultiValueVec::new();

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
            .payment((
                TokenIdentifier::from(token_id.as_str()),
                token_nonce,
                token_amount,
            ))
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn create_metastaking_pos_from_two_tokens(&mut self) {
        let token_id = String::new();
        let token_nonce = 0u64;
        let token_amount = BigUint::<StaticApi>::from(0u128);

        let metastaking_address = bech32::decode("");
        let add_liq_first_token_min_amount_out = BigUint::<StaticApi>::from(0u128);
        let add_liq_second_token_min_amount_out = BigUint::<StaticApi>::from(0u128);

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
            .payment((
                TokenIdentifier::from(token_id.as_str()),
                token_nonce,
                token_amount,
            ))
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn create_farm_staking_pos_from_single_token(&mut self) {
        let token_id = String::new();
        let token_nonce = 0u64;
        let token_amount = BigUint::<StaticApi>::from(0u128);

        let farm_staking_address = bech32::decode("");
        let min_amount_out = BigUint::<StaticApi>::from(0u128);
        // let swap_operations = MultiValueVec::from(vec![MultiValue4::from((bech32::decode(""), ManagedBuffer::new_from_bytes(&b""[..]), TokenIdentifier::from_esdt_bytes(&b""[..]), BigUint::<StaticApi>::from(0u128)))]);
        let swap_operations = MultiValueVec::new();

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
            .payment((
                TokenIdentifier::from(token_id.as_str()),
                token_nonce,
                token_amount,
            ))
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn exit_metastaking_pos_endpoint(&mut self) {
        let token_id = String::new();
        let token_nonce = 0u64;
        let token_amount = BigUint::<StaticApi>::from(0u128);

        let metastaking_address = bech32::decode("");
        let first_token_min_amount_out = BigUint::<StaticApi>::from(0u128);
        let second_token_min_amont_out = BigUint::<StaticApi>::from(0u128);

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
                TokenIdentifier::from(token_id.as_str()),
                token_nonce,
                token_amount,
            ))
            .returns(ReturnsResultUnmanaged)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn exit_farm_pos(&mut self) {
        let egld_mex_farm_address = &self.config.egld_mex_farm_address;
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
                2u64,
                BigUint::from(102693910530449043u64),
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


    async fn try_exit_lp_wrong_address_test(&mut self) {
        let egld_usdc_pair_address = &self.config.egld_usdc_pair_address;

        let first_token_min_amount_out = BigUint::<StaticApi>::from(1u128);
        let second_token_min_amont_out = BigUint::<StaticApi>::from(1u128);

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .typed(auto_pos_creator_proxy::AutoPosCreatorProxy)
            .exit_lp_pos(
                egld_usdc_pair_address,
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
}
