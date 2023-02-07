pub static ERROR_BAD_PAYMENT_TOKENS: &[u8] = b"Bad payment token";
pub static ERROR_FARM_DOES_NOT_EXIST: &[u8] = b"Farm does not exist";
pub static ERROR_FARM_ALREADY_DEFINED: &[u8] = b"Farm already defined";
pub static ERROR_FARM_HAS_FUNDS: &[u8] = b"Farm has user funds";
pub static ERROR_UNBOND_TOO_SOON: &[u8] = b"Unbonding period has not passed";
pub static ERROR_TOKEN_ROLES: &[u8] = b"Contract must already have mint & burn roles assigned";
pub static ERROR_DIVISION_CONSTANT_VALUE: &[u8] =
    b"Farm division safety constant must be greater than 0";
pub static ERROR_EXTERNAL_CONTRACT_OUTPUT: &[u8] = b"Invalid external contract output";
