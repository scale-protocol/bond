use anchor_lang::prelude::*;
use instructions::*;
declare_id!("7RErJw5JG2JgzkVsmQCXPr8SzYbZF3f2UTZCLaYEHuzE");
pub mod com;
pub mod errors;
pub mod instructions;
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
    ) -> Result<Pubkey> {
        market::initialize_market(ctx, category, spread, bump)
    }
}
