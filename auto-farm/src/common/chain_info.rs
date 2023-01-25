elrond_wasm::imports!();
elrond_wasm::derive_imports!();

static mut CURRENT_CHAIN_INFO: Option<CurrentChainInfo> = None;

#[derive(TypeAbi, TopEncode, NestedEncode, Copy, Clone)]
pub struct CurrentChainInfo {
    pub block: u64,
    pub epoch: u64,
    pub timestamp: u64,
}

impl CurrentChainInfo {
    pub fn new<Api: BlockchainApi>() -> Self {
        unsafe {
            match CURRENT_CHAIN_INFO {
                Some(cci) => cci,
                None => {
                    let api = Api::blockchain_api_impl();
                    let cci = CurrentChainInfo {
                        block: api.get_block_nonce(),
                        epoch: api.get_block_epoch(),
                        timestamp: api.get_block_timestamp(),
                    };
                    CURRENT_CHAIN_INFO = Some(cci);

                    cci
                }
            }
        }
    }
}
