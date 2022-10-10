use crate::com;
use crate::errors::BondError;
use anchor_lang::{accounts, prelude::*};
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};
use num_enum::TryFromPrimitive;
#[account]
pub struct Position {
    /// Initial position margin
    pub margin: f64,
    /// leverage size
    pub leverage: u16,
    /// 1 full position mode, 2 independent position modes.
    pub position_type: PositionType,
    /// Position status: 1 normal, 2 normal closing, 3 Forced closing, 4 pending.
    pub position_status: PositionStatus,
    /// buy long, 2 sell short.
    pub direction: Direction,
    /// Point difference data on which the quotation is based
    pub spread: f64,
    /// the position size
    pub size: f64,
    // Opening quotation (expected opening price under the listing mode)
    pub open_price: f64,
    /// Opening quotation slot
    pub open_price_slot: u64,
    /// Closing quotation
    pub close_price: f64,
    /// Closing quotation slot
    pub close_price_slot: u64,
    /// Automatic profit stop price
    pub stop_surplus_price: f64,
    /// Automatic stop loss price
    pub stop_loss_price: f64,
    /// Opening slot
    pub position_slot: u64,
    /// Order creation time
    pub create_time: i64,
    pub open_time: i64,
    pub close_time: i64,
    /// The effective time of the order.
    /// If the position is not opened successfully after this time in the order listing mode,
    /// the order will be closed directly
    pub validity_time: i64,
    /// Opening operator (the user manually, or the clearing robot in the listing mode)
    pub open_operator: Pubkey,
    /// Account number of warehouse closing operator (user manual, or clearing robot Qiangping)
    pub close_operator: Pubkey,
    /// Wallet account number of the position
    pub authority: Pubkey,
    /// Market account number of the position
    pub market_account: Pubkey,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, TryFromPrimitive)]
#[repr(u8)]
pub enum PositionType {
    Full = 1,
    Independent,
}
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, TryFromPrimitive)]
#[repr(u8)]
pub enum PositionStatus {
    Normal = 1,
    NormalClosing,
    ForceClosing,
    Pending,
}
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, TryFromPrimitive)]
#[repr(u8)]
pub enum Direction {
    Buy = 1,
    Sell,
}

impl Position {
    pub const LEN: usize = 8 + 2 + (1 + 1) * 3 + 8 * 13 + 32 * 4;
}
