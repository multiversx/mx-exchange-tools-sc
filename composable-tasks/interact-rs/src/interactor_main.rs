#![allow(non_snake_case)]

mod proxy;

use multiversx_sc_snippets::{imports::*, sdk};
use serde::{Deserialize, Serialize};
use std::{
    io::{Read, Write},
    path::Path,
};

const GATEWAY: &str = sdk::blockchain::DEVNET_GATEWAY;
const STATE_FILE: &str = "state.toml";

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
        "composeTasks" => interact.compose_tasks().await,
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
    state: State,
}

impl ContractInteract {
    async fn new() -> Self {
        let mut interactor = Interactor::new(GATEWAY).await;
        let wallet_address = interactor.register_wallet(test_wallets::alice());

        let contract_code = BytesValue::interpret_from(
            "mxsc:../output/adder.mxsc.json",
            &InterpreterContext::default(),
        );

        ContractInteract {
            interactor,
            wallet_address,
            contract_code,
            state: State::load_state(),
        }
    }

    async fn deploy(&mut self) {
        let initial_value = BigUint::<StaticApi>::from(0u128);

        let new_address = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .typed(proxy::AdderProxy)
            .init(initial_value)
            .code(&self.contract_code)
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
        let new_addr = bech32::decode("");

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .typed(proxy::AdderProxy)
            .set_wrap_egld_address(new_addr)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn set_router_address(&mut self) {
        let new_addr = bech32::decode("");

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .typed(proxy::AdderProxy)
            .set_router_address(new_addr)
            .returns(ReturnsRawResult)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }

    async fn compose_tasks(&mut self) {
        let token_id = String::new();
        let token_nonce = 0u64;
        let token_amount = BigUint::<StaticApi>::from(0u128);

        let min_expected_token_out = EgldOrEsdtTokenPayment::new(
            EgldOrEsdtTokenIdentifier::esdt(&b""[..]),
            0u64,
            BigUint::from(0u128),
        );
        let tasks = PlaceholderInput;

        let response = self
            .interactor
            .tx()
            .from(&self.wallet_address)
            .to(self.state.current_address())
            .typed(proxy::AdderProxy)
            .compose_tasks(min_expected_token_out, tasks)
            .payment((
                TokenIdentifier::from(token_id.as_str()),
                token_nonce,
                token_amount,
            ))
            .returns(ReturnsRawResult)
            .prepare_async()
            .run()
            .await;

        println!("Result: {response:?}");
    }
}
