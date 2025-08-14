multiversx_sc::imports!();

pub type Percent = u32;
pub const MAX_PERCENT: Percent = 10_000; // 100%

#[multiversx_sc::module]
pub trait CommonStorageModule {
    #[view(getRouterAddress)]
    #[storage_mapper("routerAddress")]
    fn router_address(&self) -> SingleValueMapper<ManagedAddress>;

    #[view(getTreasuryAddress)]
    #[storage_mapper("treasuryAddress")]
    fn treasury_address(&self) -> SingleValueMapper<ManagedAddress>;

    #[view(getP2pProtocolFeePercent)]
    #[storage_mapper("p2pProtocolFeePercent")]
    fn p2p_protocol_fee_percent(&self) -> SingleValueMapper<Percent>;

    #[view(getPruningFee)]
    #[storage_mapper("pruningFee")]
    fn pruning_fee(&self) -> SingleValueMapper<u8>; // TODO: Change 
}
