use crate::{
    com,
    errors::BondError,
    state::{market, position, user},
};

use anchor_lang::prelude::*;
use anchor_spl::{
    mint,
    token::{self, spl_token::instruction::AuthorityType, Mint, Token, TokenAccount, Transfer},
};
use std::convert::TryFrom;
pub fn open_position(
    ctx: Context<OpenPosition>,
    category: String,
    size: f64,
    leverage: u16,
    position_type: u8,
    direction: u8,
) -> Result<()> {
    // check parameter
    if size <= 0.0 {
        return Err(BondError::InvalidParameterOfPosition.into());
    }
    if leverage <= 0 || leverage > com::MAX_LEVERAGE as u16 {
        return Err(BondError::InvalidParameterOfPosition.into());
    }
    // set position data
    let position_account = &mut ctx.accounts.position_account;
    let market_account = &mut ctx.accounts.market_account;

    position_account.position_type =
        position::PositionType::try_from(position_type).map_err(|err| {
            msg!("{:?}", err);
            BondError::InvalidParameterOfPosition
        })?;
    position_account.direction = position::Direction::try_from(direction).map_err(|err| {
        msg!("{:?}", err);
        BondError::InvalidParameterOfPosition
    })?;
    position_account.leverage = leverage;

    let price = market_account.get_price(&mut ctx.accounts.pyth_price_account)?;
    let margin = match position_account.direction {
        position::Direction::Buy => {
            (size as f64 * price.buy / leverage as f64 * 100.0).round() / 100.0
        }
        position::Direction::Sell => {
            (size as f64 * price.sell / leverage as f64 * 100.0).round() / 100.0
        }
    };
    position_account.margin = margin;
    // check margin
    msg!("create position order by {:?}", category);
    Ok(())
}
#[derive(Accounts)]
#[instruction(category:String)]
pub struct OpenPosition<'info> {
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
    /// CHECK: Verify later
    #[account(
        constraint = market_account.pyth_price_account.key() == pyth_price_account.key()@BondError::InvalidPriceAccount,
    )]
    pub pyth_price_account: AccountInfo<'info>,
    // #[account(constraint=market_account.pyth_price_account.key()==chianlink_price_account.key()@BondError::InvalidPriceAccount)]
    // pub chianlink_price_account: AccountInfo<'info>,
    system_program: Program<'info, System>,
}
// get the equity
fn get_equity() {}

// get the profit
fn get_profit() {}
