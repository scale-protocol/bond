use anchor_lang::prelude::*;
use instructions::*;
declare_id!("ECte5vr5zJkRVnEPY9XPkgq3JFfFkthrMKxLk6gfa7v4");
pub mod com;
pub mod errors;
pub mod instructions;
pub mod price;
pub mod state;
#[program]
pub mod bond {
    use super::*;
    /// Generate system vault account
    pub fn initialize_vault(ctx: Context<InitializeVault>, bump: u8) -> Result<Pubkey> {
        vault::initialize_vault(ctx, bump)
    }
    /// create market
    pub fn initialize_market(
        ctx: Context<InitializeMarket>,
        pair: String,
        spread: f64,
        bump: u8,
        pyth_price_account: String,
        chianlink_price_account: String,
    ) -> Result<Pubkey> {
        market::initialize_market(
            ctx,
            pair,
            spread,
            bump,
            pyth_price_account,
            chianlink_price_account,
        )
    }
    pub fn initialize_user_account(ctx: Context<InitUserAccount>, bump: u8) -> Result<Pubkey> {
        user::initialize_user_account(ctx, bump)
    }
    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        user::deposit(ctx, amount)
    }
    pub fn open_position(
        ctx: Context<OpenPosition>,
        pair: String,
        size: f64,
        leverage: u16,
        position_type: u8,
        direction: u8,
    ) -> Result<()> {
        position::open_position(ctx, pair, size, leverage, position_type, direction)
    }
    pub fn close_position(ctx: Context<ClosePosition>) -> Result<()> {
        position::close_position(ctx)
    }
    pub fn investment(ctx: Context<Investment>, pair: String, amount: u64) -> Result<()> {
        market::investment(ctx, pair, amount)
    }
    pub fn divestment(ctx: Context<Divestment>, pair: String, amount: u64) -> Result<()> {
        market::divestment(ctx, pair, amount)
    }
}
