multiversx_sc::imports!();
multiversx_sc::derive_imports!();

#[derive(TypeAbi, TopEncode, TopDecode, NestedEncode, NestedDecode, Clone, Copy, PartialEq)]
pub enum DeployActionType {
    None,
    LiquidityPool,
    SimpleLock,
    ProxyDex,
    Farm,
    FarmStaking,
    Metastaking,
}
