// #![allow(non_snake_case)]

// use auto_farm::external_storage_read::farm_storage_read::ProxyTrait as _;
// use auto_farm::external_storage_read::metastaking_storage_read::ProxyTrait as _;
// use auto_farm::fees::ProxyTrait as _;
// use auto_farm::registration::ProxyTrait as _;
// use auto_farm::user_tokens::user_farm_tokens::ProxyTrait as _;
// use auto_farm::user_tokens::user_metastaking_tokens::ProxyTrait as _;
// use auto_farm::user_tokens::user_rewards::ProxyTrait as _;
// use auto_farm::whitelists::farms_whitelist::ProxyTrait as _;
// use auto_farm::whitelists::metastaking_whitelist::ProxyTrait as _;
// use auto_farm::ProxyTrait as _;
// use auto_farm::{
//     external_sc_interactions::multi_contract_interactions::ProxyTrait as _,
//     external_storage_read::farm_storage_read::FarmConfig,
// };
// use energy_query::ProxyTrait as _;
// use multiversx_sc_snippets::erdrs::wallet::Wallet;
// use multiversx_sc_snippets::multiversx_sc_scenario::scenario_format::interpret_trait::InterpreterContext;
// use multiversx_sc_snippets::multiversx_sc_scenario::scenario_model::IntoBlockchainCall;
// use multiversx_sc_snippets::{
//     env_logger,
//     multiversx_sc::{
//         codec::multi_types::*,
//         types::{Address, CodeMetadata},
//     },
//     multiversx_sc_scenario::{bech32, ContractInfo, DebugApi},
//     tokio, Interactor,
// };

// const GATEWAY: &str = multiversx_sc_snippets::erdrs::blockchain::DEVNET_GATEWAY;
// const PEM: &str = "devnetWalletKey.pem";
// const SC_ADDRESS: &str = "erd1qqqqqqqqqqqqqpgqnxsz48mu6m882qwq09dh66jxjdfm0rkk082s8r9fpp";

// const DEFAULT_ADDRESS_EXPR: &str =
//     "0x0000000000000000000000000000000000000000000000000000000000000000";
// const DEFAULT_GAS_LIMIT: u64 = 100_000_000;

// type ContractType = ContractInfo<auto_farm::Proxy<DebugApi>>;

// /// Setup steps:
// /// - deploy
// /// - addFarms (farms and farm-staking are both added through this endpoint)
// ///     No additional setup is needed, as the auto-farm SC will read the required data from the farm's storage
// /// - addMetastakingScs (optional)
// ///
// /// User actions:
// /// - register (for users who only want fees-collector/metabonding, without any farm interactions)
// /// - depositFarmTokens (deposits farm tokens, and also registers the user)
// /// - depositMetastakingTokens (similar to the one above, but for metastaking, i.e. dual yield tokens)
// /// - withdrawAllFarmTokens, withdrawSpecificFarmTokens
// /// - withdrawAllMetastakingTokens, withdrawSpecificMetastakingTokens
// /// - withdrawAllAndUnregister
// /// - userClaimRewards - claim accumulated rewards
// ///
// /// Proxy actions:
// /// - claimAllRewardsAndCompound - claims all rewards from all contracts (metabonding, fees collector, farms, etc.)
// ///     and compounds those rewards with a user's existing farm position, if they have any that fit
// ///     (i.e. farming token == reward token). These will generally be farm-staking contracts.
// /// - claimFees - on each claim, a part is taken as fees from the user. These fees can be claimed with this endpoint.

fn main() {}

// #[tokio::main]
// async fn main() {
//     env_logger::init();
//     let _ = DebugApi::dummy();

//     let mut args = std::env::args();
//     let _ = args.next();
//     let cmd = args.next().expect("at least one argument required");
//     let mut state = State::new().await;
//     match cmd.as_str() {
//         "deploy" => state.deploy().await,
//         "changeProxyClaimAddress" => state.change_proxy_claim_address().await,
//         "addFarms" => state.add_farms().await,
//         "removeFarms" => state.remove_farms().await,
//         "getFarmForFarmToken" => state.get_farm_for_farm_token_view().await,
//         "getFarmForFarmingToken" => state.get_farm_for_farming_token_view().await,
//         "getFarmConfig" => state.get_farm_config().await,
//         "register" => state.register().await,
//         "withdrawAllAndUnregister" => state.withdraw_all_and_unregister().await,
//         "depositFarmTokens" => state.deposit_farm_tokens().await,
//         "withdrawAllFarmTokens" => state.withdraw_all_farm_tokens_endpoint().await,
//         "withdrawSpecificFarmTokens" => state.withdraw_specific_farm_tokens_endpoint().await,
//         "getUserFarmTokens" => state.get_user_farm_tokens_view().await,
//         "addMetastakingScs" => state.add_metastaking_scs().await,
//         "removeMetastakingScs" => state.remove_metastaking_scs().await,
//         "getMetastakingForDualYieldToken" => {
//             state.get_metastaking_for_dual_yield_token_view().await
//         }
//         "getMetastakingForLpFarmToken" => state.get_metastaking_for_lp_farm_token().await,
//         "depositMetastakingTokens" => state.deposit_metastaking_tokens().await,
//         "withdrawAllMetastakingTokens" => state.withdraw_all_metastaking_tokens_endpoint().await,
//         "withdrawSpecificMetastakingTokens" => {
//             state.withdraw_specific_metastaking_tokens_endpoint().await
//         }
//         "getUserMetastakingTokens" => state.get_user_metastaking_tokens_view().await,
//         "getMetastakingConfig" => state.get_metastaking_config().await,
//         "claimAllRewardsAndCompound" => state.claim_all_rewards_and_compound().await,
//         "userClaimRewards" => state.user_claim_rewards_endpoint().await,
//         "getUserRewards" => state.get_user_rewards_view().await,
//         "claimFees" => state.claim_fees().await,
//         "getFeePercentage" => state.fee_percentage().await,
//         "getAccumulatedFees" => state.accumulated_fees().await,
//         "setEnergyFactoryAddress" => state.set_energy_factory_address().await,
//         "getEnergyFactoryAddress" => state.energy_factory_address().await,
//         _ => panic!("unknown command: {}", &cmd),
//     }
// }

// struct State {
//     interactor: Interactor,
//     wallet_address: Address,
//     contract: ContractType,
// }

// impl State {
//     async fn new() -> Self {
//         let mut interactor = Interactor::new(GATEWAY).await;
//         let wallet_address = interactor.register_wallet(Wallet::from_pem_file(PEM).unwrap());
//         let sc_addr_expr = if SC_ADDRESS.is_empty() {
//             DEFAULT_ADDRESS_EXPR.to_string()
//         } else {
//             "bech32:".to_string() + SC_ADDRESS
//         };
//         let contract = ContractType::new(sc_addr_expr);

//         State {
//             interactor,
//             wallet_address,
//             contract,
//         }
//     }

//     async fn deploy(&mut self) {
//         let proxy_claim_address = self.wallet_address.clone();
//         let fee_percentage = 1_000u64; // 10%
//         let energy_factory_address =
//             bech32::decode("erd1qqqqqqqqqqqqqpgqp6qrf7yp4l25c08384vgdghz7wz0f60h0n4s0m88l4");
//         let metabonding_sc_address = energy_factory_address.clone(); // not used here
//         let fees_collector_sc_address =
//             bech32::decode("erd1qqqqqqqqqqqqqpgq82pd37ra5vqnsaq5cc50ll073gzm4ahx0n4s793d9d");

//         let result: multiversx_sc_snippets::InteractorResult<()> = self
//             .interactor
//             .sc_deploy(
//                 self.contract
//                     .init(
//                         proxy_claim_address,
//                         fee_percentage,
//                         energy_factory_address,
//                         metabonding_sc_address,
//                         fees_collector_sc_address,
//                     )
//                     .into_blockchain_call()
//                     .from(&self.wallet_address)
//                     .code_metadata(CodeMetadata::all())
//                     .contract_code(
//                         "file:../output/auto-farm.wasm",
//                         &InterpreterContext::default(),
//                     )
//                     .gas_limit(DEFAULT_GAS_LIMIT),
//             )
//             .await;

//         let new_address = result.new_deployed_address();
//         let new_address_bech32 = bech32::encode(&new_address);
//         println!("new address: {}", new_address_bech32);
//     }

//     async fn change_proxy_claim_address(&mut self) {
//         let new_proxy_claim_address = PlaceholderInput;

//         let result: multiversx_sc_snippets::InteractorResult<PlaceholderOutput> = self
//             .interactor
//             .sc_call_get_result(
//                 self.contract
//                     .change_proxy_claim_address(new_proxy_claim_address)
//                     .into_blockchain_call()
//                     .from(&self.wallet_address)
//                     .gas_limit(DEFAULT_GAS_LIMIT),
//             )
//             .await;
//         let result_value = result.value();

//         println!("Result: {:?}", result_value);
//     }

//     async fn add_farms(&mut self) {
//         let mut farms = MultiValueVec::new();
//         farms.push(bech32::decode(
//             "erd1qqqqqqqqqqqqqpgq6wrtdnv7d5uypnaw2k8mtujfdl0s66t40n4sag5e7n",
//         ));
//         farms.push(bech32::decode(
//             "erd1qqqqqqqqqqqqqpgqv6j2vr8tr9rc0fwhu0s2xef9w64qww2h0n4ssmxgq0",
//         ));
//         farms.push(bech32::decode(
//             "erd1qqqqqqqqqqqqqpgq5dzs6yf47tnsk5ays2aedzmu2ahsmcqv0n4s3rsljy",
//         ));

//         let _: multiversx_sc_snippets::InteractorResult<()> = self
//             .interactor
//             .sc_call_get_result(
//                 self.contract
//                     .add_farms(farms)
//                     .into_blockchain_call()
//                     .from(&self.wallet_address)
//                     .gas_limit(DEFAULT_GAS_LIMIT),
//             )
//             .await;
//     }

//     async fn remove_farms(&mut self) {
//         let farms = PlaceholderInput;

//         let result: multiversx_sc_snippets::InteractorResult<PlaceholderOutput> = self
//             .interactor
//             .sc_call_get_result(
//                 self.contract
//                     .remove_farms(farms)
//                     .into_blockchain_call()
//                     .from(&self.wallet_address)
//                     .gas_limit(DEFAULT_GAS_LIMIT),
//             )
//             .await;
//         let result_value = result.value();

//         println!("Result: {:?}", result_value);
//     }

//     async fn get_farm_for_farm_token_view(&mut self) {
//         // let farm_token_id = PlaceholderInput;

//         // let result_value: PlaceholderOutput = self
//         //     .interactor
//         //     .vm_query(self.contract.get_farm_for_farm_token_view(farm_token_id))
//         //     .await;

//         // println!("Result: {:?}", result_value);
//     }

//     async fn get_farm_for_farming_token_view(&mut self) {
//         // let farming_token_id = PlaceholderInput;

//         // let result_value: PlaceholderOutput = self
//         //     .interactor
//         //     .vm_query(
//         //         self.contract
//         //             .get_farm_for_farming_token_view(farming_token_id),
//         //     )
//         //     .await;

//         // println!("Result: {:?}", result_value);
//     }

//     async fn get_farm_config(&mut self) {
//         let farm_address =
//             bech32::decode("erd1qqqqqqqqqqqqqpgq6wrtdnv7d5uypnaw2k8mtujfdl0s66t40n4sag5e7n");

//         let result_value: FarmConfig<DebugApi> = self
//             .interactor
//             .vm_query(self.contract.get_farm_config(farm_address))
//             .await;

//         println!("Result: {:?}", result_value);
//     }

//     async fn register(&mut self) {
//         let result: multiversx_sc_snippets::InteractorResult<PlaceholderOutput> = self
//             .interactor
//             .sc_call_get_result(
//                 self.contract
//                     .register()
//                     .into_blockchain_call()
//                     .from(&self.wallet_address)
//                     .gas_limit(DEFAULT_GAS_LIMIT),
//             )
//             .await;
//         let result_value = result.value();

//         println!("Result: {:?}", result_value);
//     }

//     async fn withdraw_all_and_unregister(&mut self) {
//         let result: multiversx_sc_snippets::InteractorResult<PlaceholderOutput> = self
//             .interactor
//             .sc_call_get_result(
//                 self.contract
//                     .withdraw_all_and_unregister()
//                     .into_blockchain_call()
//                     .from(&self.wallet_address)
//                     .gas_limit(DEFAULT_GAS_LIMIT),
//             )
//             .await;
//         let result_value = result.value();

//         println!("Result: {:?}", result_value);
//     }

//     async fn deposit_farm_tokens(&mut self) {
//         let token_id = b"";
//         let token_nonce = 0u64;
//         let token_amount = 0u64;

//         let result: multiversx_sc_snippets::InteractorResult<PlaceholderOutput> = self
//             .interactor
//             .sc_call_get_result(
//                 self.contract
//                     .deposit_farm_tokens()
//                     .into_blockchain_call()
//                     .from(&self.wallet_address)
//                     .esdt_transfer(token_id.to_vec(), token_nonce, token_amount)
//                     .gas_limit(DEFAULT_GAS_LIMIT),
//             )
//             .await;
//         let result_value = result.value();

//         println!("Result: {:?}", result_value);
//     }

//     async fn withdraw_all_farm_tokens_endpoint(&mut self) {
//         let result: multiversx_sc_snippets::InteractorResult<PlaceholderOutput> = self
//             .interactor
//             .sc_call_get_result(
//                 self.contract
//                     .withdraw_all_farm_tokens_endpoint()
//                     .into_blockchain_call()
//                     .from(&self.wallet_address)
//                     .gas_limit(DEFAULT_GAS_LIMIT),
//             )
//             .await;
//         let result_value = result.value();

//         println!("Result: {:?}", result_value);
//     }

//     async fn withdraw_specific_farm_tokens_endpoint(&mut self) {
//         let tokens_to_withdraw = PlaceholderInput;

//         let result: multiversx_sc_snippets::InteractorResult<PlaceholderOutput> = self
//             .interactor
//             .sc_call_get_result(
//                 self.contract
//                     .withdraw_specific_farm_tokens_endpoint(tokens_to_withdraw)
//                     .into_blockchain_call()
//                     .from(&self.wallet_address)
//                     .gas_limit(DEFAULT_GAS_LIMIT),
//             )
//             .await;
//         let result_value = result.value();

//         println!("Result: {:?}", result_value);
//     }

//     async fn get_user_farm_tokens_view(&mut self) {
//         let user = PlaceholderInput;

//         let result_value: PlaceholderOutput = self
//             .interactor
//             .vm_query(self.contract.get_user_farm_tokens_view(user))
//             .await;

//         println!("Result: {:?}", result_value);
//     }

//     async fn add_metastaking_scs(&mut self) {
//         let scs = PlaceholderInput;

//         let result: multiversx_sc_snippets::InteractorResult<PlaceholderOutput> = self
//             .interactor
//             .sc_call_get_result(
//                 self.contract
//                     .add_metastaking_scs(scs)
//                     .into_blockchain_call()
//                     .from(&self.wallet_address)
//                     .gas_limit(DEFAULT_GAS_LIMIT),
//             )
//             .await;
//         let result_value = result.value();

//         println!("Result: {:?}", result_value);
//     }

//     async fn remove_metastaking_scs(&mut self) {
//         let scs = PlaceholderInput;

//         let result: multiversx_sc_snippets::InteractorResult<PlaceholderOutput> = self
//             .interactor
//             .sc_call_get_result(
//                 self.contract
//                     .remove_metastaking_scs(scs)
//                     .into_blockchain_call()
//                     .from(&self.wallet_address)
//                     .gas_limit(DEFAULT_GAS_LIMIT),
//             )
//             .await;
//         let result_value = result.value();

//         println!("Result: {:?}", result_value);
//     }

//     async fn get_metastaking_for_dual_yield_token_view(&mut self) {
//         // let dual_yield_token_id = PlaceholderInput;

//         // let result_value: PlaceholderOutput = self
//         //     .interactor
//         //     .vm_query(
//         //         self.contract
//         //             .get_metastaking_for_dual_yield_token_view(dual_yield_token_id),
//         //     )
//         //     .await;

//         // println!("Result: {:?}", result_value);
//     }

//     async fn get_metastaking_for_lp_farm_token(&mut self) {
//         // let lp_farm_token_id = PlaceholderInput;

//         // let result_value: PlaceholderOutput = self
//         //     .interactor
//         //     .vm_query(
//         //         self.contract
//         //             .get_metastaking_for_lp_farm_token(lp_farm_token_id),
//         //     )
//         //     .await;

//         // println!("Result: {:?}", result_value);
//     }

//     async fn deposit_metastaking_tokens(&mut self) {
//         let token_id = b"";
//         let token_nonce = 0u64;
//         let token_amount = 0u64;

//         let result: multiversx_sc_snippets::InteractorResult<PlaceholderOutput> = self
//             .interactor
//             .sc_call_get_result(
//                 self.contract
//                     .deposit_metastaking_tokens()
//                     .into_blockchain_call()
//                     .from(&self.wallet_address)
//                     .esdt_transfer(token_id.to_vec(), token_nonce, token_amount)
//                     .gas_limit(DEFAULT_GAS_LIMIT),
//             )
//             .await;
//         let result_value = result.value();

//         println!("Result: {:?}", result_value);
//     }

//     async fn withdraw_all_metastaking_tokens_endpoint(&mut self) {
//         let result: multiversx_sc_snippets::InteractorResult<PlaceholderOutput> = self
//             .interactor
//             .sc_call_get_result(
//                 self.contract
//                     .withdraw_all_metastaking_tokens_endpoint()
//                     .into_blockchain_call()
//                     .from(&self.wallet_address)
//                     .gas_limit(DEFAULT_GAS_LIMIT),
//             )
//             .await;
//         let result_value = result.value();

//         println!("Result: {:?}", result_value);
//     }

//     async fn withdraw_specific_metastaking_tokens_endpoint(&mut self) {
//         let tokens_to_withdraw = PlaceholderInput;

//         let result: multiversx_sc_snippets::InteractorResult<PlaceholderOutput> = self
//             .interactor
//             .sc_call_get_result(
//                 self.contract
//                     .withdraw_specific_metastaking_tokens_endpoint(tokens_to_withdraw)
//                     .into_blockchain_call()
//                     .from(&self.wallet_address)
//                     .gas_limit(DEFAULT_GAS_LIMIT),
//             )
//             .await;
//         let result_value = result.value();

//         println!("Result: {:?}", result_value);
//     }

//     async fn get_user_metastaking_tokens_view(&mut self) {
//         let user = PlaceholderInput;

//         let result_value: PlaceholderOutput = self
//             .interactor
//             .vm_query(self.contract.get_user_metastaking_tokens_view(user))
//             .await;

//         println!("Result: {:?}", result_value);
//     }

//     async fn get_metastaking_config(&mut self) {
//         let metastaking_address = PlaceholderInput;

//         let result_value: PlaceholderOutput = self
//             .interactor
//             .vm_query(self.contract.get_metastaking_config(metastaking_address))
//             .await;

//         println!("Result: {:?}", result_value);
//     }

//     async fn claim_all_rewards_and_compound(&mut self) {
//         let claim_args = PlaceholderInput;

//         let result: multiversx_sc_snippets::InteractorResult<PlaceholderOutput> = self
//             .interactor
//             .sc_call_get_result(
//                 self.contract
//                     .claim_all_rewards_and_compound(claim_args)
//                     .into_blockchain_call()
//                     .from(&self.wallet_address)
//                     .gas_limit(DEFAULT_GAS_LIMIT),
//             )
//             .await;
//         let result_value = result.value();

//         println!("Result: {:?}", result_value);
//     }

//     async fn user_claim_rewards_endpoint(&mut self) {
//         let result: multiversx_sc_snippets::InteractorResult<PlaceholderOutput> = self
//             .interactor
//             .sc_call_get_result(
//                 self.contract
//                     .user_claim_rewards_endpoint()
//                     .into_blockchain_call()
//                     .from(&self.wallet_address)
//                     .gas_limit(DEFAULT_GAS_LIMIT),
//             )
//             .await;
//         let result_value = result.value();

//         println!("Result: {:?}", result_value);
//     }

//     async fn get_user_rewards_view(&mut self) {
//         let user = PlaceholderInput;

//         let result_value: PlaceholderOutput = self
//             .interactor
//             .vm_query(self.contract.get_user_rewards_view(user))
//             .await;

//         println!("Result: {:?}", result_value);
//     }

//     async fn claim_fees(&mut self) {
//         let result: multiversx_sc_snippets::InteractorResult<PlaceholderOutput> = self
//             .interactor
//             .sc_call_get_result(
//                 self.contract
//                     .claim_fees()
//                     .into_blockchain_call()
//                     .from(&self.wallet_address)
//                     .gas_limit(DEFAULT_GAS_LIMIT),
//             )
//             .await;
//         let result_value = result.value();

//         println!("Result: {:?}", result_value);
//     }

//     async fn fee_percentage(&mut self) {
//         let result_value: PlaceholderOutput = self
//             .interactor
//             .vm_query(self.contract.fee_percentage())
//             .await;

//         println!("Result: {:?}", result_value);
//     }

//     async fn accumulated_fees(&mut self) {
//         let result_value: PlaceholderOutput = self
//             .interactor
//             .vm_query(self.contract.accumulated_fees())
//             .await;

//         println!("Result: {:?}", result_value);
//     }

//     async fn set_energy_factory_address(&mut self) {
//         let sc_address = PlaceholderInput;

//         let result: multiversx_sc_snippets::InteractorResult<PlaceholderOutput> = self
//             .interactor
//             .sc_call_get_result(
//                 self.contract
//                     .set_energy_factory_address(sc_address)
//                     .into_blockchain_call()
//                     .from(&self.wallet_address)
//                     .gas_limit(DEFAULT_GAS_LIMIT),
//             )
//             .await;
//         let result_value = result.value();

//         println!("Result: {:?}", result_value);
//     }

//     async fn energy_factory_address(&mut self) {
//         let result_value: PlaceholderOutput = self
//             .interactor
//             .vm_query(self.contract.energy_factory_address())
//             .await;

//         println!("Result: {:?}", result_value);
//     }
// }
