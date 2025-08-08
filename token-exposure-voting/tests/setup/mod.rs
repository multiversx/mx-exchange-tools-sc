#![allow(deprecated)]

use energy_factory::SimpleLockEnergy;
use multiversx_sc::{
    storage::mappers::StorageTokenWrapper,
    types::{Address, EsdtLocalRole, MultiValueEncoded},
};
use multiversx_sc_modules::pause::PauseModule;
use multiversx_sc_scenario::{
    managed_address, managed_biguint, managed_token_id, rust_biguint, testing_framework::*,
    DebugApi,
};
use simple_lock::locked_token::LockedTokenModule;
use token_exposure_voting::{config::ConfigModule, vote::VoteModule, TokenExposureVotingModule};
use week_timekeeping::WeekTimekeepingModule;

pub static VOTING_TOKEN_ID: &[u8] = b"VOTE-123456";
pub static TEST_TOKENS: &[&[u8]] = &[
    b"TOKEN-01",
    b"TOKEN-02",
    b"TOKEN-03",
    b"TOKEN-04",
    b"TOKEN-05",
    b"TOKEN-06",
    b"TOKEN-07",
    b"TOKEN-08",
    b"TOKEN-09",
    b"TOKEN-10",
];
pub static REWARD_TOKEN_ID: &[u8] = b"MEX-123456";
pub static LOCKED_TOKEN_ID: &[u8] = b"LOCKED-123456";
pub static LEGACY_LOCKED_TOKEN_ID: &[u8] = b"LEGACY-123456";
pub const FIRST_WEEK_START_EPOCH: u64 = 100;
pub const BOOST_AMOUNT: u64 = 1_000_000_000; // 1 VOTE token
pub const EPOCHS_IN_YEAR: u64 = 365;
pub static LOCK_OPTIONS: &[u64] = &[EPOCHS_IN_YEAR, 2 * EPOCHS_IN_YEAR, 4 * EPOCHS_IN_YEAR];
pub static PENALTY_PERCENTAGES: &[u64] = &[4_000, 6_000, 8_000];

pub struct TokenExposureVotingSetup<ScBuilder, EnergyBuilder>
where
    ScBuilder: 'static + Copy + Fn() -> token_exposure_voting::ContractObj<DebugApi>,
    EnergyBuilder: 'static + Copy + Fn() -> energy_factory::ContractObj<DebugApi>,
{
    pub blockchain: BlockchainStateWrapper,
    pub owner: Address,
    pub _energy_factory_wrapper:
        ContractObjWrapper<energy_factory::ContractObj<DebugApi>, EnergyBuilder>,
    pub sc_wrapper: ContractObjWrapper<token_exposure_voting::ContractObj<DebugApi>, ScBuilder>,
}

impl<ScBuilder, EnergyBuilder> TokenExposureVotingSetup<ScBuilder, EnergyBuilder>
where
    ScBuilder: 'static + Copy + Fn() -> token_exposure_voting::ContractObj<DebugApi>,
    EnergyBuilder: 'static + Copy + Fn() -> energy_factory::ContractObj<DebugApi>,
{
    pub fn new(sc_builder: ScBuilder, energy_builder: EnergyBuilder) -> Self {
        let mut blockchain = BlockchainStateWrapper::new();
        let owner = blockchain.create_user_account(&rust_biguint!(0));

        // Create energy factory contract
        let energy_factory_wrapper = blockchain.create_sc_account(
            &rust_biguint!(0),
            Some(&owner),
            energy_builder,
            "energy factory",
        );

        // Initialize the energy factory
        blockchain
            .execute_tx(&owner, &energy_factory_wrapper, &rust_biguint!(0), |sc| {
                let mut lock_options = MultiValueEncoded::new();
                for (option, penalty) in LOCK_OPTIONS.iter().zip(PENALTY_PERCENTAGES.iter()) {
                    lock_options.push((*option, *penalty).into());
                }

                sc.init(
                    managed_token_id!(REWARD_TOKEN_ID),
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

        // Set up necessary token roles for energy factory
        blockchain.set_esdt_local_roles(
            energy_factory_wrapper.address_ref(),
            REWARD_TOKEN_ID,
            &[EsdtLocalRole::Mint, EsdtLocalRole::Burn],
        );
        blockchain.set_esdt_local_roles(
            energy_factory_wrapper.address_ref(),
            LOCKED_TOKEN_ID,
            &[
                EsdtLocalRole::NftCreate,
                EsdtLocalRole::NftAddQuantity,
                EsdtLocalRole::NftBurn,
                EsdtLocalRole::Transfer,
            ],
        );
        blockchain.set_esdt_local_roles(
            energy_factory_wrapper.address_ref(),
            LEGACY_LOCKED_TOKEN_ID,
            &[EsdtLocalRole::NftBurn],
        );

        // Create the main contract
        let sc_wrapper = blockchain.create_sc_account(
            &rust_biguint!(0),
            Some(&owner),
            sc_builder,
            "token exposure voting",
        );

        // Initialize the main contract
        blockchain
            .execute_tx(&owner, &sc_wrapper, &rust_biguint!(0), |sc| {
                sc.init(
                    FIRST_WEEK_START_EPOCH,
                    managed_address!(energy_factory_wrapper.address_ref()),
                    managed_token_id!(VOTING_TOKEN_ID),
                );
            })
            .assert_ok();

        Self {
            blockchain,
            owner,
            _energy_factory_wrapper: energy_factory_wrapper,
            sc_wrapper,
        }
    }

    pub fn create_user_with_voting_tokens(&mut self, balance: u64) -> Address {
        let user = self.blockchain.create_user_account(&rust_biguint!(0));
        self.blockchain
            .set_esdt_balance(&user, VOTING_TOKEN_ID, &rust_biguint!(balance));
        user
    }

    pub fn set_current_week(&mut self, week_offset: u64) {
        let epoch = FIRST_WEEK_START_EPOCH + week_offset * 7 * 24 * 60 * 10;
        self.blockchain.set_block_epoch(epoch);
    }

    pub fn boost_token(&mut self, user: &Address, token_id: &[u8], amount: u64) {
        self.blockchain
            .execute_esdt_transfer(
                user,
                &self.sc_wrapper,
                VOTING_TOKEN_ID,
                0,
                &rust_biguint!(amount),
                |sc| {
                    sc.boost(managed_token_id!(token_id));
                },
            )
            .assert_ok();
    }

    pub fn setup_tokens_for_week(&mut self, week: usize, tokens: &[&[u8]]) {
        self.blockchain
            .execute_tx(&self.owner, &self.sc_wrapper, &rust_biguint!(0), |sc| {
                for token in tokens {
                    sc.tokens_for_week(week).insert(managed_token_id!(*token));
                }
            })
            .assert_ok();
    }

    pub fn set_token_votes(&mut self, token_id: &[u8], week: usize, votes: u64) {
        self.blockchain
            .execute_tx(&self.owner, &self.sc_wrapper, &rust_biguint!(0), |sc| {
                sc.token_votes(&managed_token_id!(token_id), week)
                    .set(managed_biguint!(votes));
            })
            .assert_ok();
    }

    pub fn withdraw_boost_funds_as_owner(&mut self) {
        self.blockchain
            .execute_tx(&self.owner, &self.sc_wrapper, &rust_biguint!(0), |sc| {
                sc.withdraw_boost_funds();
            })
            .assert_ok();
    }

    pub fn get_current_week(&mut self) -> usize {
        let mut week = 0;
        self.blockchain
            .execute_query(&self.sc_wrapper, |sc| {
                week = sc.get_current_week();
            })
            .assert_ok();
        week
    }

    pub fn check_contract_balance(&self, expected_balance: u64) {
        let balance =
            self.blockchain
                .get_esdt_balance(self.sc_wrapper.address_ref(), VOTING_TOKEN_ID, 0);
        assert_eq!(balance, rust_biguint!(expected_balance));
    }

    pub fn check_user_balance(&self, user: &Address, expected_balance: u64) {
        let balance = self.blockchain.get_esdt_balance(user, VOTING_TOKEN_ID, 0);
        assert_eq!(balance, rust_biguint!(expected_balance));
    }
}
