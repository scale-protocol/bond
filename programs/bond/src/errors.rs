use anchor_lang::error_code;

#[error_code]
pub enum BondError {
    #[msg("Category does not exceed 20 bytes")]
    CategoryTooLong,
    #[msg("invalid full position market")]
    InvalidFullPositionMarket,
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
    #[msg("Risk control, it is not allowed to open new positions")]
    RiskControlBlocking,
    #[msg("Insufficient margin available")]
    InsufficientMargin,
    #[msg("The market does not support opening this type of position")]
    MarketNotSupportOpenPosition,
    #[msg("The market pauses to open new positions")]
    MarketPauses,
}
