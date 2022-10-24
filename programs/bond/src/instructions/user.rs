use crate::com;
use crate::errors::BondError;
use crate::state::user::*;
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};

pub fn initialize_user_account(ctx: Context<InitAccount>) -> Result<Pubkey> {
    let account = &mut ctx.accounts.user_account;
    account.authority = ctx.accounts.initializer.key();
    // Reserve the next order number forever
    account.position_seed_offset = 1;
    account.balance = 0.0;
    account.profit = 0.0;
    account.margin_total = 0.0;
    account.margin_full_total = 0.0;
    account.margin_independent_total = 0.0;
    Ok(ctx.accounts.user_account.key())
}
#[derive(Accounts)]
pub struct InitAccount<'info> {
    #[account(mut)]
    pub initializer: Signer<'info>,
    #[account(
        init,
        payer=initializer,
        space= 8+UserAccount::LEN,
        seeds = [com::USER_ACCOUNT_SEED,initializer.key().as_ref()],
        bump,
    )]
    pub user_account: Account<'info, UserAccount>,
    system_program: Program<'info, System>,
}

pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
    // transfer
    token::transfer(ctx.accounts.into(), amount)?;
    let user_account = &mut ctx.accounts.user_account;
    let balance = amount as f64;
    user_account.balance = balance;
    Ok(())
}

#[derive(Accounts)]
#[instruction(amount:u64)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(address=com::base_account::get_vault_mint())]
    pub token_mint: Account<'info, Mint>,
    #[account(
        mut,
        constraint=user_token_account.amount >= amount@BondError::InsufficientBalance,
    )]
    pub user_token_account: Account<'info, TokenAccount>,
    #[account(
        mut,
        has_one = authority@BondError::UserTransactionAccountMismatch,
        seeds = [com::USER_ACCOUNT_SEED,authority.key().as_ref()],
        bump,
    )]
    pub user_account: Account<'info, UserAccount>,
    #[account(
        mut,
        token::mint=token_mint,
        seeds = [com::VAULT_TOKEN_ACCOUNT_SEED],
        bump,
        )]
    pub vault_token_account: Account<'info, TokenAccount>,
    token_program: Program<'info, Token>,
}

impl<'info> From<&mut Deposit<'info>> for CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
    fn from(accounts: &mut Deposit<'info>) -> Self {
        let cpi_accounts = Transfer {
            from: accounts.user_token_account.to_account_info().clone(),
            to: accounts.vault_token_account.to_account_info().clone(),
            authority: accounts.authority.to_account_info().clone(),
        };
        let cpi_program = accounts.token_program.to_account_info().clone();
        CpiContext::new(cpi_program, cpi_accounts)
    }
}
