use multiversx_sc_scenario::imports::*;

const OWNER: TestAddress = TestAddress::new("owner");
const USER: TestAddress = TestAddress::new("user");

const COMPOSABLE_TASKS_ADDRESS: TestSCAddress = TestSCAddress::new("composable_tasks");
const WEGLD_ADDRESS: TestSCAddress = TestSCAddress::new("wegld");
const ROUTER_ADDRESS: TestSCAddress = TestSCAddress::new("router");
const PAIR1_ADDRESS: TestSCAddress = TestSCAddress::new("pair1");
const PAIR2_ADDRESS: TestSCAddress = TestSCAddress::new("pair2");

const BALANCE: u64 = 100_000_000;
const FIRST_TOKEN_ID: TestTokenIdentifier = TestTokenIdentifier::new("FIRST-123456");
const SECOND_TOKEN_ID: TestTokenIdentifier = TestTokenIdentifier::new("SECOND-123456");
const WEGLD_TOKEN_ID: TestTokenIdentifier = TestTokenIdentifier::new("WEGLD-123456");

const COMPOSABLE_TASKS_PATH: FilePath = FilePath("output/composable-tasks.wasm");
const WEGLD_PATH: FilePath = FilePath("tests/test_contracts/multiversx-wegld-swap-sc.wasm");
const ROUTER_PATH: FilePath = FilePath("tests/test_contracts/router.wasm");
const PAIR_PATH: FilePath = FilePath("tests/test_contracts/pair.wasm");

fn world() -> ScenarioWorld {
    let mut blockchain = ScenarioWorld::new();

    blockchain.register_contract(COMPOSABLE_TASKS_PATH, composable_tasks::ContractBuilder);
    blockchain.register_contract(WEGLD_PATH, multiversx_wegld_swap_sc::ContractBuilder);
    blockchain.register_contract(PAIR_PATH, pair::ContractBuilder);
    blockchain.register_contract(ROUTER_PATH, router::ContractBuilder);

    blockchain
}

struct ComposableTasksTestState {
    world: ScenarioWorld,
}

impl ComposableTasksTestState {
    fn new() -> Self {
        let mut world = world();
        world.start_trace();

        world.account(OWNER).nonce(1);
        world
            .account(USER)
            .nonce(1)
            .balance(BALANCE)
            .esdt_balance(FIRST_TOKEN_ID, BALANCE)
            .esdt_balance(WEGLD_TOKEN_ID, BALANCE)
            .esdt_balance(SECOND_TOKEN_ID, BALANCE);

        Self { world }
    }

    fn deploy_composable_tasks_contract(&mut self) -> &mut Self {
        self.world
            .tx()
            .from(OWNER)
            .raw_deploy()
            .code(COMPOSABLE_TASKS_PATH)
            .new_address(COMPOSABLE_TASKS_ADDRESS)
            .run();

        self
    }

    fn deploy_wegld_contract(&mut self) -> &mut Self {
        self.world
            .tx()
            .from(OWNER)
            .raw_deploy()
            .code(WEGLD_PATH)
            .argument(&WEGLD_TOKEN_ID)
            .new_address(WEGLD_ADDRESS)
            .run();

        self
    }

    fn deploy_pair_contract(
        &mut self,
        first_token_id: TestTokenIdentifier,
        address: TestSCAddress,
    ) -> &mut Self {
        self.world
            .tx()
            .from(OWNER)
            .raw_deploy()
            .code(PAIR_PATH)
            .argument(&first_token_id)
            .argument(&WEGLD_TOKEN_ID)
            .argument(&OWNER)
            .argument(&OWNER)
            .argument(&0u64)
            .argument(&0u64)
            .argument(&ManagedAddress::<StaticApi>::zero())
            .new_address(address)
            .run();

        self
    }

    fn deploy_pair_contracts(&mut self) -> &mut Self {
        self.deploy_pair_contract(FIRST_TOKEN_ID, PAIR1_ADDRESS);
        self.deploy_pair_contract(SECOND_TOKEN_ID, PAIR2_ADDRESS)
    }

    fn deploy_router_contract(&mut self) -> &mut Self {
        self.world
            .tx()
            .from(OWNER)
            .raw_deploy()
            .code(ROUTER_PATH)
            .argument(&ManagedAddress::<StaticApi>::zero())
            .new_address(ROUTER_ADDRESS)
            .run();

        self
    }
}

#[test]
fn test_deploy_composable_tasks() {
    let mut state = ComposableTasksTestState::new();
    state.deploy_composable_tasks_contract();
    state.deploy_wegld_contract();
    state.deploy_pair_contracts();
    state.deploy_router_contract();
}
