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
) -> Result<()> {
    let position_account = &mut ctx.accounts.position_account;
    let market_account = &mut ctx.accounts.market_account;

    let price = market_account.get_price(&mut ctx.accounts.pyth_price_account)?;
    let margin = match position_account.direction {
        position::Direction::Buy => {}
        position::Direction::Sell => {}
    };

    msg!("create position order by {:?}", category);
    Ok(())
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
        seeds=[com::POSITION_ACCOUNT_SEED,authority.key().as_ref(),market_account.key().as_ref(),user_account.position_seed_offset.to_string().as_bytes().as_ref()],
        bump,
    )]
    pub position_account: Account<'info, position::Position>,
    #[account(
        seeds=[com::POSITION_INDEX_ACCOUNT_SEED,authority.key().as_ref()],bump
    )]
    pub position_index_account: Account<'info, user::PositionIndexAccount>,
    /// CHECK: Verify later
    #[account(
        constraint = market_account.pyth_price_account.key() == pyth_price_account.key()@BondError::InvalidPriceAccount,
    )]
    pub pyth_price_account: AccountInfo<'info>,
    // #[account(constraint=market_account.pyth_price_account.key()==chianlink_price_account.key()@BondError::InvalidPriceAccount)]
    // pub chianlink_price_account: AccountInfo<'info>,
    system_program: Program<'info, System>,
}
