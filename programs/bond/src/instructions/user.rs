use crate::com;
use crate::errors::BondError;
use crate::state::market;
use crate::state::user;
use anchor_lang::prelude::*;
use anchor_spl::{
    mint,
    token::{self, spl_token::instruction::AuthorityType, Mint, Token, TokenAccount, Transfer},
};

pub fn initialize_user_account(ctx: Context<UserAccount>, bump: u8) -> Result<Pubkey> {
    let account = &mut ctx.accounts.user_account;
    account.authority = ctx.accounts.initializer.key();
    account.position_seed_offset = 0;
    account.balance = 0.0;
    account.profit = 0.0;
    account.margin_total = 0.0;
    account.margin_full_total = 0.0;
    account.margin_independent_total = 0.0;
    msg!("bump:{:?}", bump);
    Ok(ctx.accounts.user_account.key())
}
#[derive(Accounts)]
#[instruction(bump:u8)]
pub struct UserAccount<'info> {
    #[account(mut)]
    pub initializer: Signer<'info>,
    #[account(
        init,
        payer=initializer,
        space=user::UserAccount::LEN + 8,
        seeds = [b"scale_user_account",initializer.key().as_ref()],
        bump,
    )]
    pub user_account: Account<'info, user::UserAccount>,
    system_program: Program<'info, System>,
}

pub fn deposit(ctx: Context<Deposit>, amount: u64, category: String) -> Result<()> {
    let user_account = &mut ctx.accounts.user_account;
    let market_account = &mut ctx.accounts.market_account;
    // transfer
    let rs = token::transfer(ctx.accounts.into(), amount);
    match rs {
        Ok(x) => x,
        Err(err) => {
            msg!("err====>{:?}", err)
        }
    }
    msg!("category:{:?}", category);
    Ok(())
}

#[derive(Accounts)]
#[instruction(amount:u64,category:String)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    // #[account(address=mint::USDC)]
    // #[account(address=com::get_vault_mint())]
    pub token_mint: Account<'info, Mint>,
    #[account(
        mut,
        constraint=user_token_account.amount >= amount@BondError::InsufficientBalance,
    )]
    pub user_token_account: Account<'info, TokenAccount>,
    #[account(mut,has_one = authority@BondError::UserTransactionAccountMismatch)]
    pub user_account: Account<'info, user::UserAccount>,
    #[account(
        mut,
        token::mint=token_mint,
        seeds = [b"vault_token"],
        bump,
        )]
    pub vault_token_account: Account<'info, TokenAccount>,
    #[account(
        mut,
        constraint=market_account.category == category@BondError::IllegalMarketAccount,
    )]
    pub market_account: Account<'info, market::Market>,
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