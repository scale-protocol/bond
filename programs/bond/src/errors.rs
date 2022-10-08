use anchor_lang::error_code;

#[error_code]
pub enum BondError {
    #[msg("Category does not exceed 20 bytes")]
    CategoryTooLong,
}
