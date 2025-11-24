multiversx_sc::imports!();
multiversx_sc::derive_imports!();

pub static INVALID_TOKEN_SENT_ERR_MSG: &[u8] = b"Invalid token sent";
pub static SENT_TOO_FEW_TOKENS_ERR_MSG: &[u8] = b"Sent too few tokens";

pub struct ProcessP2pFillArgs<'a, M: ManagedTypeApi> {
    pub maker: &'a ManagedAddress<M>,
    pub input_token: TokenIdentifier<M>,
    pub min_maker_amount: BigUint<M>,
    pub taker: &'a ManagedAddress<M>,
    pub tokens_to_buy: BigUint<M>,
    pub taker_payment: &'a EsdtTokenPayment<M>,
}

#[type_abi]
#[derive(TopEncode, TopDecode)]
pub struct GetTokensBoughtP2pResultType<M: ManagedTypeApi> {
    pub tokens_bought: BigUint<M>,
    pub taker_token_amount: BigUint<M>,
    pub fees: BigUint<M>,
    pub surplus: BigUint<M>,
}

pub enum ProcessP2pFillResult {
    Fail,
    Success,
}
