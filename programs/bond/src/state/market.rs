use crate::com;
use crate::price::price;
use crate::state::position;
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
    /// Total amount of outstanding NFT bonds.
    pub vault_full: u64,
    /// Token balance of basic current fund.
    pub vault_base_balance: f64,
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
    pub pyth_price_account: Pubkey,
    pub chianlink_price_account: Pubkey,
    /// Transaction category (token type, such as BTC, ETH)
    /// len: 4+20
    pub category: String,
    /// Point difference (can be understood as slip point),
    /// deviation between the executed quotation and the actual quotation
    pub spread: f64,
    /// Market operator, 1 project party, other marks to be defined
    pub officer: bool,
    /// Whether full position  mode is supported
    pub is_support_full_position: bool,
}
pub struct Price {
    pub buy_price: f64,
    pub sell_price: f64,
    pub real_price: f64,
    pub spread: f64,
}

impl Market {
    pub const LEN: usize =
        2 + 8 * 4 + (1 + 1) + 8 * 6 + 32 + (32 * 5) + 32 * 2 + (4 + 20) + 8 + 1 + 1;
    // get current price
    pub fn get_price(
        &self,
        price_account_info_pyth: &AccountInfo,
        _price_account_info_chinalink: &AccountInfo,
    ) -> Result<Price> {
        let p = price::get_price(price_account_info_pyth)?;
        let spread = com::f64_round(p * self.spread);
        Ok(Price {
            buy_price: com::f64_round(p + spread),
            sell_price: com::f64_round(p - spread),
            real_price: p,
            spread: spread,
        })
    }
    pub fn get_exposure(&self) -> f64 {
        (self.long_position_total.abs() - self.short_position_total.abs()).abs()
    }

    pub fn get_total_liquidity(&self) -> f64 {
        self.vault_base_balance + self.vault_profit_balance
    }

    pub fn get_exposure_proportion(&self) -> f64 {
        let exposure = self.get_exposure();
        if exposure == 0.0 {
            return 0.0;
        }
        exposure / self.get_total_liquidity()
    }

    pub fn get_fund_rate(&self) -> f64 {
        self.get_exposure_proportion() * com::FUND_RATE
    }

    pub fn get_current_dominant_direction(&self) -> position::Direction {
        if self.long_position_total > self.short_position_total {
            return position::Direction::Buy;
        }
        position::Direction::Sell
    }

    pub fn get_position_fund(
        &self,
        direction: position::Direction,
        position_fund_size: f64,
    ) -> f64 {
        if direction == self.get_current_dominant_direction() {
            return -(position_fund_size * self.get_fund_rate());
        }
        let max = self.long_position_total.max(self.short_position_total);
        let min = self.long_position_total.min(self.short_position_total);
        let total_deducted_funds = max * self.get_fund_rate();
        // Total funds allocated to disadvantaged positions
        let total_funds_allocated = total_deducted_funds * (min / max);
        total_funds_allocated * (position_fund_size / min)
    }
}
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, TryFromPrimitive, PartialEq)]
#[repr(u8)]
pub enum MarketStatus {
    Normal = 1,
    Locked,
    Frozen,
}
