use anchor_lang::prelude::*;
use num_enum::TryFromPrimitive;
#[account]
pub struct Market {
    /// Maximum allowable leverage ratio
    pub max_leverage: u16,
    /// position management rate
    pub management_rate: f64,
    /// transaction rate
    pub transaction_rate: f64,
    /// insurance rate
    pub insurance_rate: f64,
    /// margin rate,Current constant positioning 100%
    pub margin_rate: f64,
    /// Market status:
    /// 1 Normal;
    /// 2. Lock the market, allow closing settlement and not open positions;
    /// 3 The market is frozen, and opening and closing positions are not allowed.
    pub status: MarketStatus,
    /// Token balance of basic current fund.
    pub vault_balance: f64,
    /// Total amount of outstanding NFT bonds.
    pub vault_full: u64,
    /// Token balance of profit and loss fund
    pub vault_profit_balance: f64,
    /// Insurance fund token balance
    pub vault_insurance_balance: f64,
    /// Total amount of long positions in the market
    pub long_position_total: f64,
    /// Total amount of short positions in the market
    pub short_position_total: f64,
    /// Market administrator account address
    pub authority: Pubkey,
    /// Market operator address, with authority to operate rate, up to 5 can be set.
    pub operator: [Pubkey; 5],
    /// Transaction category (token type, such as BTC, ETH)
    /// len: 4+20
    pub category: String,
    /// Point difference (can be understood as slip point),
    /// deviation between the executed quotation and the actual quotation
    pub spread: f64,
    /// Market operator, 1 project party, other marks to be defined
    pub officer: u16,
}

impl Market {
    pub const LEN: usize =
        2 + 8 + 8 + 8 + 8 + (1 + 1) + 8 + 8 + 8 + 8 + 8 + 8 + 32 + (32 * 5) + (4 + 20) + 8 + 2;
}
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, TryFromPrimitive)]
#[repr(u8)]
pub enum MarketStatus {
    Normal = 1,
    Locked,
    Frozen,
}
