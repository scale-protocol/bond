use crate::com;
use crate::errors::BondError;
use crate::state::market;
use crate::state::position;
use crate::state::user;
use anchor_lang::prelude::*;
use anchor_spl::{
    mint,
    token::{self, spl_token::instruction::AuthorityType, Mint, Token, TokenAccount, Transfer},
};
pub fn create_position(
    ctx: Context<CreatePosition>,
    category: String,
    size: f64,
    leverage: u64,
    position_type: u8,
    direction: u8,
) {
}
#[derive(Accounts)]
#[instruction(category:String)]
pub struct CreatePosition<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        mut,
        has_one = authority@BondError::UserTransactionAccountMismatch,
        seeds = [com::USER_ACCOUNT_SEED,authority.key().as_ref()],
        bump,
    )]
    pub user_account: Account<'info, user::UserAccount>,
    #[account(
        mut,
        constraint=market_account.category == category@BondError::IllegalMarketAccount,
        seeds = [com::MARKET_ACCOUNT_SEED,category.as_bytes()],
        bump,
    )]
    pub market_account: Account<'info, market::Market>,
    #[account(
        init,
        payer=authority,
        space=position::Position::LEN+8,
        seeds=[com::POSITION_ACCOUNT_SEED,authority.key().as_ref(),market_account.key().as_ref(),user_account.position_seed_offset.to_be_bytes().as_ref()],
        bump,
    )]
    pub position_account: Account<'info, position::Position>,
    system_program: Program<'info, System>,
}
