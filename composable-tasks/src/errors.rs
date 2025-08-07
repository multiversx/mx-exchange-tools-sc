pub static ERROR_CANNOT_SWAP_EGLD: &[u8] = b"EGLD can't be swapped!";
pub static ERROR_WRONG_PAYMENT_TOKEN_NOT_EGLD: &[u8] = b"Payment token is not EGLD!";
pub static ERROR_SMART_SWAP_ARGUMENTS: &[u8] = b"Smart swap invalid arguments";
pub static ERROR_SMART_SWAP_TOO_MANY_OPERATIONS: &[u8] = b"Smart swap too many operations";
pub static ERROR_MISSING_NUMBER_OPS: &[u8] = b"Missing number of operations";
pub static ERROR_INVALID_NUMBER_OPS: &[u8] = b"Invalid number of operations";
pub static ERROR_MISSING_AMOUNT_IN: &[u8] = b"Missing partial amount_in";
pub static ERROR_WRONG_PERCENTAGE_AMOUNT: &[u8] = b"Wrong percentage amount";
pub static ERROR_MISSING_NUMBER_SWAP_OPS: &[u8] = b"Missing number of swap operations";
pub static ERROR_INVALID_NUMBER_SWAP_OPS: &[u8] = b"Invalid number of swap operations";
pub static ERROR_MISSING_PAIR_ADDR: &[u8] = b"Missing pair address";
pub static ERROR_MISSING_FUNCTION_NAME: &[u8] = b"Missing function";
pub static ERROR_INVALID_FUNCTION_NAME: &[u8] = b"Invalid function name";
pub static ERROR_MISSING_TOKEN_ID: &[u8] = b"Missing token ID";
pub static ERROR_INVALID_TOKEN_ID: &[u8] = b"Invalid token ID";
pub static ERROR_MISSING_AMOUNT: &[u8] = b"Missing amount";
pub static ERROR_ACC_AMOUNT_EXCEEDS_PAYMENT_IN: &[u8] =
    b"Accumulated amount_in exceeds task input payment";
pub static ERROR_ROUTER_SWAP_0_PAYMENTS: &[u8] = b"Router swap returned 0 payments";
pub static ERROR_INVALID_NUMBER_ROUTER_SWAP_ARGS: &[u8] =
    b"Invalid number of router swap arguments";
pub static ERROR_INCORRECT_ARGS: &[u8] = b"Incorrect arguments";
pub static ERROR_BACK_TRANSFERS_WRONG_PAYMENTS_NO: &[u8] =
    b"Wrong back transfers expected no of payments";
pub static ERROR_WRONG_RETURNED_TOKEN_IDENTIFIER: &[u8] = b"Wrong returned token identifier!";
pub static ERROR_INVALID_PERCENTAGE: &[u8] = b"Invalid percentage value";
pub static ERROR_ZERO_AMOUNT: &[u8] = b"Amount must be greater than zero";
