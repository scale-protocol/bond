use crate::errors::BondError;
use crate::state::market;
use crate::{accounts, com};
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};
pub fn initialize_market(
    ctx: Context<InitializeMarket>,
    category: String,
    spread: f64,
    bump: u8,
    pyth_price_account: String,
    chianlink_price_account: String,
) -> Result<Pubkey> {
    let market_account = &mut ctx.accounts.market_account;
    if category.as_bytes().len() > 20 {
        return err!(BondError::CategoryTooLong);
    }
    market_account.category = category.clone();
    market_account.max_leverage = 125;
    market_account.management_rate = 0.0004;
    market_account.transaction_rate = 0.003;
    market_account.insurance_rate = 0.0005;
    market_account.margin_rate = 1.0;
    market_account.status = market::MarketStatus::Normal;
    market_account.vault_full = 0;
    market_account.vault_base_balance = 0.0;
    market_account.vault_profit_balance = 0.0;
    market_account.vault_insurance_balance = 0.0;
    market_account.long_position_total = 0.0;
    market_account.short_position_total = 0.0;
    market_account.authority = ctx.accounts.initializer.key();
    market_account.operator = [ctx.accounts.initializer.key(); 5];
    market_account.spread = spread;
    market_account.officer = false;
    market_account.is_support_full_position = false;
    if ctx.accounts.initializer.key() == com::base_account::get_team_authority() {
        market_account.officer = true;
        let c = com::FullPositionMarket::from(category.as_str());
        if c != com::FullPositionMarket::None {
            market_account.is_support_full_position = true;
        }
    }
    market_account.pyth_price_account =
        Pubkey::try_from(pyth_price_account.as_str()).map_err(|err| {
            msg!("invalid pubkey error:{:?}", err);
            BondError::InvalidPubkey
        })?;
    market_account.chianlink_price_account = Pubkey::try_from(chianlink_price_account.as_str())
        .map_err(|err| {
            msg!("invalid pubkey error:{:?}", err);
            BondError::InvalidPubkey
        })?;
    msg!("bump:{:?}", bump);
    Ok(ctx.accounts.market_account.key())
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
    pub market_account: Account<'info, market::Market>,
    system_program: Program<'info, System>,
}

pub fn investment(ctx: Context<Investment>, category: String, amount: u64) -> Result<()> {
    token::transfer(ctx.accounts.into(), amount)?;
    let market_account = &mut ctx.accounts.market_account;
    market_account.vault_full += amount;
    market_account.vault_base_balance += amount as f64;
    msg!("investment category:{:?}", category);
    Ok(())
}
#[derive(Accounts)]
#[instruction(category: String,amount:u64)]
pub struct Investment<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    // #[account(address=com::get_vault_mint())]
    pub token_mint: Account<'info, Mint>,
    #[account(
        mut,
        constraint=user_token_account.amount >= amount@BondError::InsufficientBalance,
    )]
    pub user_token_account: Account<'info, TokenAccount>,
    #[account(
        mut,
        token::mint=token_mint,
        seeds = [com::VAULT_TOKEN_ACCOUNT_SEED],
        bump,
        )]
    pub vault_token_account: Account<'info, TokenAccount>,
    #[account(
        mut,
        seeds = [com::MARKET_ACCOUNT_SEED,category.as_bytes()],
        bump,
    )]
    pub market_account: Box<Account<'info, market::Market>>,
    pub token_program: Program<'info, Token>,
}

impl<'info> From<&mut Investment<'info>> for CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
    fn from(accounts: &mut Investment<'info>) -> Self {
        let cpi_accounts = Transfer {
            from: accounts.user_token_account.to_account_info().clone(),
            to: accounts.vault_token_account.to_account_info().clone(),
            authority: accounts.user.to_account_info().clone(),
        };
        let cpi_program = accounts.token_program.to_account_info().clone();
        CpiContext::new(cpi_program, cpi_accounts)
    }
}

pub fn divestment(ctx: Context<Divestment>, category: String, amount: u64) -> Result<()> {
    let cpi_ctx: CpiContext<Transfer> = ctx.accounts.into();

    let (_pda, bump_seed) =
        Pubkey::find_program_address(&[com::VAULT_TOKEN_AUTHORITY_SEED], ctx.program_id);
    let seeds = &[&com::VAULT_TOKEN_AUTHORITY_SEED[..], &[bump_seed]];
    token::transfer(cpi_ctx.with_signer(&[&seeds[..]]), amount)?;

    let market_account = &mut ctx.accounts.market_account;
    if market_account.vault_full < amount {
        return Err(BondError::InsufficientVaultBalance.into());
    }
    market_account.vault_full -= amount;
    market_account.vault_base_balance -= amount as f64;
    msg!("divestment category:{:?}", category);

    Ok(())
}

#[derive(Accounts)]
#[instruction(category: String,amount:u64)]
pub struct Divestment<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    // #[account(address=com::get_vault_mint())]
    pub token_mint: Account<'info, Mint>,
    #[account(
        mut,
        constraint=vault_token_account.owner != user.key()@BondError::NoPermission,
        constraint=vault_token_account.amount >= amount@BondError::InsufficientVaultBalance,
    )]
    pub user_token_account: Account<'info, TokenAccount>,
    #[account(
        mut,
        token::mint=token_mint,
        seeds = [com::VAULT_TOKEN_ACCOUNT_SEED],
        bump,
        )]
    pub vault_token_account: Account<'info, TokenAccount>,
    /// CHECK: non check
    #[account(
        seeds = [com::VAULT_TOKEN_AUTHORITY_SEED],
        bump,
    )]
    pub pda_authority_account: AccountInfo<'info>,
    #[account(
        mut,
        seeds = [com::MARKET_ACCOUNT_SEED,category.as_bytes()],
        bump,
    )]
    pub market_account: Box<Account<'info, market::Market>>,
    pub token_program: Program<'info, Token>,
}

impl<'info> From<&mut Divestment<'info>> for CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
    fn from(accounts: &mut Divestment<'info>) -> Self {
        let cpi_accounts = Transfer {
            from: accounts.vault_token_account.to_account_info().clone(),
            to: accounts.user_token_account.to_account_info().clone(),
            authority: accounts.pda_authority_account.to_account_info().clone(),
        };
        let cpi_program = accounts.token_program.to_account_info().clone();
        CpiContext::new(cpi_program, cpi_accounts)
    }
}
