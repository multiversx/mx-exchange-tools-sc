elrond_wasm::imports!();
elrond_wasm::derive_imports!();

#[derive(TypeAbi, TopEncode, NestedEncode)]
pub struct CurrentChainInfo {
    pub block: u64,
    pub epoch: u64,
    pub timestamp: u64,
}

impl CurrentChainInfo {
    pub fn new<Api: BlockchainApi>() -> Self {
        let api = Api::blockchain_api_impl();
        CurrentChainInfo {
            block: api.get_block_nonce(),
            epoch: api.get_block_epoch(),
            timestamp: api.get_block_timestamp(),
        }
    }
}
