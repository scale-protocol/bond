use crate::com;
use crate::errors::BondError;
use anchor_lang::{accounts, prelude::*};
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};
#[account]
pub struct UserAccount {
    /// Account owner wallet address
    pub authority: Pubkey,
    /// The position offset.
    /// This value is increased by one each time the position is opened to determine
    ///  the PDA account number of the position (this value can be used as the order number).
    pub position_seed_offset: u64,
    /// Balance of user account (maintain the deposit,
    ///  and the balance here will be deducted when the deposit used in the full position mode is deducted)
    pub balance: f64,
    /// User settled profit
    pub profit: f64,
    /// Total amount of deposit used.
    pub margin_total: f64,
    /// Total amount of used margin in full warehouse mode.
    pub margin_full_total: f64,
    /// Total amount of used margin in independent position mode.
    pub margin_independent_total: f64,
}

impl UserAccount {
    pub const LEN: usize = 32 + 8 + 8 + 8 + 8 + 8 + 8;
}
