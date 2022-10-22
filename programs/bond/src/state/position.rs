use crate::com::*;
use crate::state::market;
use anchor_lang::prelude::*;
use num_enum::TryFromPrimitive;
#[account]
#[derive(Debug)]
pub struct Position {
    pub position_seed_offset: u32,
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
    /// the position size
    pub size: f64,
    /// default is 1,Reserved in the future
    pub lot: u64,
    // Opening quotation (expected opening price under the listing mode)
    pub open_price: f64,
    /// Point difference data on which the quotation is based
    pub open_spread: f64,
    // Actual quotation currently obtained
    pub open_real_price: f64,
    /// Closing quotation
    pub close_price: f64,
    /// Point difference data on which the quotation is based
    pub close_spread: f64,
    // Actual quotation currently obtained
    pub close_real_price: f64,
    // PL
    pub profit: f64,
    /// Automatic profit stop price
    pub stop_surplus_price: f64,
    /// Automatic stop loss price
    pub stop_loss_price: f64,
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

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, TryFromPrimitive, PartialEq)]
#[repr(u8)]
pub enum PositionType {
    Full = 1,
    Independent,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, TryFromPrimitive, PartialEq)]
#[repr(u8)]
pub enum PositionStatus {
    Normal = 1,
    NormalClosing,
    ForceClosing,
    Pending,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, TryFromPrimitive, PartialEq, Copy)]
#[repr(u8)]
pub enum Direction {
    Buy = 1,
    Sell,
}
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct PositionHeader {
    pub position_seed_offset: u32,
    pub open_price: f64,
    pub direction: Direction,
    pub size: f64,
    pub market: FullPositionMarket,
}

impl PositionHeader {
    pub const LEN: usize = 4 + 8 + (1 + 1) + 8 + (1 + 1);
    // Floating P/L
    pub fn get_pl_price(&self, p: &market::Price) -> f64 {
        match self.direction {
            Direction::Buy => (p.sell_price - self.open_price) * self.size,
            Direction::Sell => (self.open_price - p.buy_price) * self.size,
        }
    }
    pub fn get_fund_size(&self) -> f64 {
        self.open_price * self.size
    }
}

impl Position {
    pub const LEN: usize = 4 + 8 + 2 + (1 + 1) * 3 + 8 * 15 + 32 * 4;
    // Floating P/L
    pub fn get_pl_price(&self, p: market::Price) -> f64 {
        match self.direction {
            Direction::Buy => {
                f64_round((p.sell_price - self.open_price) * self.lot as f64 * self.size)
            }
            Direction::Sell => {
                f64_round((self.open_price - p.buy_price) * self.lot as f64 * self.size)
            }
        }
    }
    pub fn get_fund_size(&self) -> f64 {
        self.open_price * self.lot as f64 * self.size
    }
}
