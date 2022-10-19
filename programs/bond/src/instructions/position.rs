use crate::{
    com,
    errors::BondError,
    state::{market, position, user},
};

use anchor_lang::{prelude::*, solana_program::slot_history::Slot};
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
    let position_account = &mut ctx.accounts.position_account;
    let market_account = &mut ctx.accounts.market_account;
    let user_account = &mut ctx.accounts.user_account;

    if market_account.status != market::MarketStatus::Normal {
        return Err(BondError::MarketPauses.into());
    }
    // set position data
    position_account.position_type =
        position::PositionType::try_from(position_type).map_err(|err| {
            msg!("{:?}", err);
            BondError::InvalidParameterOfPosition
        })?;

    if position_account.position_type == position::PositionType::Full {
        if !market_account.is_support_full_position {
            return Err(BondError::MarketNotSupportOpenPosition.into());
        }
        if user_account.open_full_position_headers.len() >= user::MAX_OPEN_FULL_POSITION_SET_SIZE {
            return Err(BondError::FullPositionExceededLimit.into());
        }
    }
    position_account.direction = position::Direction::try_from(direction).map_err(|err| {
        msg!("{:?}", err);
        BondError::InvalidParameterOfPosition
    })?;
    position_account.leverage = leverage;

    let price = market_account.get_price(
        &ctx.accounts.pyth_price_account,
        &ctx.accounts.chianlink_price_account,
    )?;
    let margin = match position_account.direction {
        position::Direction::Buy => com::f64_round(
            size as f64 * price.buy_price / leverage as f64 * market_account.margin_rate,
        ),
        position::Direction::Sell => com::f64_round(
            size as f64 * price.sell_price / leverage as f64 * market_account.margin_rate,
        ),
    };
    position_account.position_seed_offset = user_account.position_seed_offset;
    position_account.margin = margin;
    position_account.position_status = position::PositionStatus::Normal;
    position_account.spread = price.spread;
    position_account.current_real_price = price.real_price;
    position_account.size = size;
    position_account.lot = 1;
    position_account.open_price = match position_account.direction {
        position::Direction::Buy => price.buy_price,
        position::Direction::Sell => price.sell_price,
    };
    position_account.close_price = 0.0;
    position_account.stop_surplus_price = 0.0;
    position_account.stop_loss_price = 0.0;
    position_account.create_time = Clock::get().unwrap().unix_timestamp;
    position_account.open_time = Clock::get().unwrap().unix_timestamp;
    position_account.close_time = 0;
    position_account.validity_time = 0;
    position_account.open_operator = ctx.accounts.authority.key();
    position_account.authority = ctx.accounts.authority.key();
    position_account.market_account = market_account.key();
    // --finish set position data

    let fund_size = position_account.get_fund_size();
    // set market data
    match position_account.direction {
        position::Direction::Buy => {
            market_account.long_position_total += fund_size;
            user_account.position_full_vector += 1;
        }
        position::Direction::Sell => {
            market_account.short_position_total += fund_size;
            user_account.position_full_vector -= 1;
        }
    };
    // Pay insurance fund
    let insurance_fund = margin * market_account.insurance_rate;
    market_account.vault_insurance_balance += insurance_fund;
    user_account.balance -= insurance_fund;
    // set user account data
    let position_seed_offset = user_account.position_seed_offset;
    user_account.update_index_by_open(position_seed_offset);
    user_account.add_position_header(position::PositionHeader {
        position_seed_offset,
        open_price: position_account.open_price,
        direction: position_account.direction,
        size,
        market: com::FullPositionMarket::from(category.as_str()),
    })?;
    // this is next position offset number
    user_account.position_seed_offset += 1;
    // pay margin fund
    user_account.margin_total += margin;
    match position_account.position_type {
        position::PositionType::Full => {
            user_account.margin_full_total += margin;
            match position_account.direction {
                position::Direction::Buy => user_account.margin_full_buy_total += margin,
                position::Direction::Sell => user_account.margin_full_sell_total += margin,
            }
        }
        position::PositionType::Independent => {
            user_account.margin_independent_total += margin;
            user_account.balance -= margin;
            match position_account.direction {
                position::Direction::Buy => user_account.margin_independent_buy_total += margin,
                position::Direction::Sell => user_account.margin_independent_sell_total += margin,
            }
        }
    }
    if user_account.balance < 0.0 {
        return Err(BondError::InsufficientBalanceForUser.into());
    }
    // user_account.open_full_position_headers.len()
    let exposure = market_account.get_exposure();
    let total_liquidity = market_account.get_total_liquidity();
    let fund_pool = match position_account.direction {
        position::Direction::Buy => market_account.long_position_total,
        position::Direction::Sell => market_account.short_position_total,
    };
    let margin_full_total = com::f64_round(
        user_account
            .margin_full_buy_total
            .max(user_account.margin_full_sell_total),
    );
    let user_account_equity = get_equity(ctx)?;
    // check margin
    let margin_ratio = user_account_equity / margin_full_total;
    if margin_ratio < com::BURST_RATE {
        return Err(BondError::InsufficientMargin.into());
    }
    // Risk judgment
    if exposure > total_liquidity * com::POSITION_DIFF_PROPORTION {
        return Err(BondError::RiskControlBlocking.into());
    }
    if fund_size > total_liquidity * com::POSITION_PROPORTION_ONE {
        return Err(BondError::RiskControlBlocking.into());
    }
    if fund_pool > total_liquidity * com::POSITION_PROPORTION {
        return Err(BondError::RiskControlBlocking.into());
    }
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
    /// CHECK: Verify later
    #[account(constraint=market_account.pyth_price_account.key()==chianlink_price_account.key()@BondError::InvalidPriceAccount)]
    pub chianlink_price_account: AccountInfo<'info>,
    /// CHECK: Verify later
    #[account(
        constraint=market_account_btc.category == "BTC/USD"@BondError::IllegalMarketAccount,
        constraint=market_account_btc.officer == true@BondError::IllegalMarketAccount,
    )]
    pub market_account_btc: Account<'info, market::Market>,
    /// CHECK: Verify later
    #[account(
        constraint=market_account_eth.category == "ETH/USD"@BondError::IllegalMarketAccount,
        constraint=market_account_eth.officer == true@BondError::IllegalMarketAccount,
    )]
    pub market_account_eth: Account<'info, market::Market>,
    /// CHECK: Verify later
    #[account(
        constraint=market_account_sol.category == "SOL/USD"@BondError::IllegalMarketAccount,
        constraint=market_account_sol.officer == true@BondError::IllegalMarketAccount,
    )]
    pub market_account_sol: Account<'info, market::Market>,
    /// CHECK: Verify later
    #[account(
            constraint = com::base_account::get_pyth_price_account_btc() == pyth_price_account_btc.key()@BondError::InvalidPriceAccount,
        )]
    pub pyth_price_account_btc: AccountInfo<'info>,
    /// CHECK: Verify later
    #[account(
            constraint = com::base_account::get_pyth_price_account_eth() == pyth_price_account_eth.key()@BondError::InvalidPriceAccount,
        )]
    pub pyth_price_account_eth: AccountInfo<'info>,
    /// CHECK: Verify later
    #[account(
            constraint = com::base_account::get_pyth_price_account_sol() == pyth_price_account_sol.key()@BondError::InvalidPriceAccount,
        )]
    pub pyth_price_account_sol: AccountInfo<'info>,
    /// CHECK: Verify later
    #[account(
        constraint = com::base_account::get_chainlink_price_account_btc() == chainlink_price_account_btc.key()@BondError::InvalidPriceAccount,
    )]
    pub chainlink_price_account_btc: AccountInfo<'info>,
    /// CHECK: Verify later
    #[account(
        constraint = com::base_account::get_chainlink_price_account_eth() == chainlink_price_account_eth.key()@BondError::InvalidPriceAccount,
    )]
    pub chainlink_price_account_eth: AccountInfo<'info>,
    /// CHECK: Verify later
    #[account(
        constraint = com::base_account::get_chainlink_price_account_sol() == chainlink_price_account_sol.key()@BondError::InvalidPriceAccount,
    )]
    pub chainlink_price_account_sol: AccountInfo<'info>,
    system_program: Program<'info, System>,
}
// get the equity
fn get_equity(ctx: Context<OpenPosition>) -> Result<f64> {
    let account_balance = ctx.accounts.user_account.balance;
    let total_pl = get_pl_price_all_full_position(ctx)?;
    Ok(account_balance + total_pl)
}

// Floating P/L
pub fn get_pl_price_all_full_position(ctx: Context<OpenPosition>) -> Result<f64> {
    let btc_price = ctx.accounts.market_account_btc.get_price(
        &ctx.accounts.pyth_price_account_btc,
        &ctx.accounts.chainlink_price_account_btc,
    )?;
    let eth_price = ctx.accounts.market_account_btc.get_price(
        &ctx.accounts.pyth_price_account_eth,
        &ctx.accounts.chainlink_price_account_eth,
    )?;
    let sol_price = ctx.accounts.market_account_btc.get_price(
        &ctx.accounts.pyth_price_account_sol,
        &ctx.accounts.chainlink_price_account_sol,
    )?;
    let headers = &ctx.accounts.user_account.open_full_position_headers;
    let mut total_pl: f64 = 0.0;
    for header in headers.iter() {
        let profit_and_fund_rate: f64 = match header.market {
            com::FullPositionMarket::BtcUsd => {
                header.get_pl_price(&btc_price)
                    + ctx
                        .accounts
                        .market_account_btc
                        .get_position_fund(header.direction.clone(), header.get_fund_size())
            }

            com::FullPositionMarket::EthUsd => {
                header.get_pl_price(&eth_price)
                    + ctx
                        .accounts
                        .market_account_eth
                        .get_position_fund(header.direction.clone(), header.get_fund_size())
            }

            com::FullPositionMarket::SolUsd => {
                header.get_pl_price(&sol_price)
                    + ctx
                        .accounts
                        .market_account_sol
                        .get_position_fund(header.direction.clone(), header.get_fund_size())
            }

            _ => 0.0,
        };
        total_pl += profit_and_fund_rate
    }
    Ok(total_pl)
}
