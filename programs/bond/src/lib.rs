use anchor_lang::prelude::*;
use instructions::*;
declare_id!("3CuC9qc7ehNu3MrGrqDMu6it2g71dFJTKn7184sb1TuJ");
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
        category: String,
        spread: f64,
        bump: u8,
        pyth_price_account: String,
        chianlink_price_account: String,
    ) -> Result<Pubkey> {
        market::initialize_market(
            ctx,
            category,
            spread,
            bump,
            pyth_price_account,
            chianlink_price_account,
        )
    }
    pub fn initialize_user_account(ctx: Context<InitAccount>) -> Result<Pubkey> {
        user::initialize_user_account(ctx)
    }
    pub fn deposit(ctx: Context<Deposit>, amount: u64, category: String) -> Result<()> {
        user::deposit(ctx, amount, category)
    }
    pub fn open_position(
        ctx: Context<OpenPosition>,
        category: String,
        size: f64,
        leverage: u16,
        position_type: u8,
        direction: u8,
    ) -> Result<()> {
        position::open_position(ctx, category, size, leverage, position_type, direction)
    }
    pub fn close_position(ctx: Context<ClosePosition>) -> Result<()> {
        position::close_position(ctx)
    }
    pub fn investment(ctx: Context<Investment>, category: String, amount: u64) -> Result<()> {
        market::investment(ctx, category, amount)
    }
    pub fn divestment(ctx: Context<Divestment>, category: String, amount: u64) -> Result<()> {
        market::divestment(ctx, category, amount)
    }
}
