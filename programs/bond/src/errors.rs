use anchor_lang::error_code;

#[error_code]
pub enum BondError {
    #[msg("Category does not exceed 20 bytes")]
    CategoryTooLong,
    #[msg("User transaction account mismatch")]
    UserTransactionAccountMismatch,
    #[msg("Insufficient user token account balance")]
    InsufficientBalance,
    #[msg("Illegal market account")]
    IllegalMarketAccount,
    #[msg("Invalid pubkey")]
    InvalidPubkey,
    #[msg("can not get price from pyth.network")]
    GetPriceFailedFromPyth,
    #[msg("can not get price from chainlink")]
    GetPriceFailedFromChainLink,
    #[msg("invalid price account")]
    InvalidPriceAccount,
    #[msg("Illegal instruction parameter, please check it")]
    InvalidParameterOfPosition,
}
