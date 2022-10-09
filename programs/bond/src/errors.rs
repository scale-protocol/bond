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
}
