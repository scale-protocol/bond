use crate::com;
use crate::errors::BondError;
use crate::state::market;
use anchor_lang::prelude::*;

pub fn initialize_market(
    ctx: Context<InitializeMarket>,
    category: String,
    spread: f64,
    bump: u8,
    pyth_price_account: String,
    chianlink_price_account: String,
) -> Result<Pubkey> {
    let market_data = &mut ctx.accounts.market_data;
    if category.as_bytes().len() > 20 {
        return err!(BondError::CategoryTooLong);
    }
    market_data.category = category.clone();
    market_data.max_leverage = 125;
    market_data.management_rate = 0.0004;
    market_data.transaction_rate = 0.003;
    market_data.insurance_rate = 0.0005;
    market_data.margin_level = 1.0;
    market_data.status = market::MarketStatus::Normal;
    market_data.vault_full = 0;
    market_data.vault_base_balance = 0.0;
    market_data.vault_profit_balance = 0.0;
    market_data.vault_insurance_balance = 0.0;
    market_data.long_position_total = 0.0;
    market_data.short_position_total = 0.0;
    market_data.authority = ctx.accounts.initializer.key();
    market_data.operator = [ctx.accounts.initializer.key(); 5];
    market_data.spread = spread;
    market_data.officer = false;
    market_data.is_support_full_position = false;
    if ctx.accounts.initializer.key() == com::base_account::get_team_authority() {
        market_data.officer = true;
        let c = com::FullPositionMarket::from(category.as_str());
        if c != com::FullPositionMarket::None {
            market_data.is_support_full_position = true;
        }
    }
    market_data.pyth_price_account =
        Pubkey::try_from(pyth_price_account.as_str()).map_err(|err| {
            msg!("invalid pubkey error:{:?}", err);
            BondError::InvalidPubkey
        })?;
    market_data.chianlink_price_account = Pubkey::try_from(chianlink_price_account.as_str())
        .map_err(|err| {
            msg!("invalid pubkey error:{:?}", err);
            BondError::InvalidPubkey
        })?;
    msg!("bump:{:?}", bump);
    Ok(ctx.accounts.market_data.key())
}
#[derive(Accounts)]
#[instruction(category: String,spread: f64,bump: u8)]
pub struct InitializeMarket<'info> {
    #[account(mut)]
    pub initializer: Signer<'info>,
    #[account(
        init,
        payer=initializer,
        space=market::Market::LEN + 8,
        seeds = [com::MARKET_ACCOUNT_SEED,category.as_bytes()],
        bump,
    )]
    pub market_data: Account<'info, market::Market>,
    system_program: Program<'info, System>,
}
