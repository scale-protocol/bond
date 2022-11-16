use anchor_lang::error_code;

#[error_code]
pub enum BondError {
    #[msg("Pair does not exceed 20 bytes")]
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
    #[msg(
        "Risk control,The exposure ratio should not exceed 70% of the current pool,So as to avoid the risk that the platform's current pool is empty."
    )]
    RiskControlBlockingExposure,
    #[msg(
        "Risk control,The size of a single position shall not be greater than 20% of the exposure"
    )]
    RiskControlBlockingFundSize,
    #[msg(
        "Risk control,The proportion of unidirectional positions shall not exceed 150% of the flow pool,So as to avoid the risk of malicious position opening."
    )]
    RiskControlBlockingFundPool,
    #[msg("Insufficient margin available")]
    InsufficientMargin,
    #[msg("Insufficient balance to open a new position")]
    InsufficientBalanceForUser,
    #[msg("The market does not support opening this type of position")]
    MarketNotSupportOpenPosition,
    #[msg("The market pauses to open new positions")]
    MarketPauses,
    #[msg("The market has been frozen and cannot be settled.")]
    MarketFrozen,
    #[msg("The position in this mode has exceeded the limit")]
    FullPositionExceededLimit,
    #[msg("Account number does not match")]
    AccountNumberNotMatch,
    #[msg("No permission to perform this operation")]
    NoPermission,
    #[msg("The balance of the treasury is insufficient, and the withdrawal operation cannot be performed temporarily")]
    InsufficientVaultBalance,
    #[msg("The position status is not supported")]
    PositionStatusInvalid,
}
