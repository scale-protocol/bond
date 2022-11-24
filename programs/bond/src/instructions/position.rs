use crate::{
    com,
    errors::BondError,
    state::{market, position, user},
};

use anchor_lang::prelude::*;

use std::convert::TryFrom;
pub fn open_position(
    ctx: Context<OpenPosition>,
    pair: String,
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
    let pre_exposure = market_account.get_exposure();
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
    msg!("price:{:?}", price);
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
    position_account.open_spread = price.spread;
    position_account.open_real_price = price.real_price;
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
        margin,
        market: com::FullPositionMarket::from(pair.as_str()),
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
            if user_account.balance < margin {
                return Err(BondError::InsufficientMargin.into());
            }
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

    // Risk judgment
    msg!(
        "exposure: {},total_liquidity: {},pre_exposure: {:?},position_direction: {:?}",
        exposure,
        total_liquidity * com::POSITION_DIFF_PROPORTION,
        pre_exposure,
        position_account.direction
    );
    if exposure > total_liquidity * com::POSITION_DIFF_PROPORTION && pre_exposure <= exposure {
        return Err(BondError::RiskControlBlockingExposure.into());
    }

    let user_account_equity = get_equity(ctx)?;
    // check margin
    if (user_account_equity / margin_full_total) < com::BURST_RATE {
        return Err(BondError::InsufficientMargin.into());
    }

    if fund_size > total_liquidity * com::POSITION_PROPORTION_ONE {
        return Err(BondError::RiskControlBlockingFundSize.into());
    }
    if fund_pool > total_liquidity * com::POSITION_PROPORTION {
        return Err(BondError::RiskControlBlockingFundPool.into());
    }
    msg!("create position order by {:?}", pair);
    Ok(())
}
#[derive(Accounts)]
#[instruction(pair:String)]
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
        constraint=market_account.pair == pair@BondError::IllegalMarketAccount,
        seeds = [com::MARKET_ACCOUNT_SEED,pair.as_bytes()],
        bump,
    )]
    pub market_account: Box<Account<'info, market::Market>>,
    #[account(
        init,
        payer=authority,
        space=position::Position::LEN+8,
        seeds=[com::POSITION_ACCOUNT_SEED,authority.key().as_ref(),user_account.key().as_ref(),user_account.position_seed_offset.to_string().as_bytes().as_ref()],
        bump,
    )]
    pub position_account: Account<'info, position::Position>,
    /// CHECK: Verify later
    #[account(
        constraint = market_account.pyth_price_account.key() == pyth_price_account.key()@BondError::InvalidPriceAccount,
    )]
    pub pyth_price_account: AccountInfo<'info>,
    /// CHECK: Verify later
    #[account(constraint=market_account.chianlink_price_account.key()==chianlink_price_account.key()@BondError::InvalidPriceAccount)]
    pub chianlink_price_account: AccountInfo<'info>,
    /// CHECK: Verify later
    #[account(
        constraint=market_account_btc.pair == com::FullPositionMarket::BtcUsd.to_string()@BondError::IllegalMarketAccount,
        constraint=market_account_btc.officer == true@BondError::IllegalMarketAccount,
    )]
    pub market_account_btc: Box<Account<'info, market::Market>>,
    /// CHECK: Verify later
    #[account(
        constraint=market_account_eth.pair == com::FullPositionMarket::EthUsd.to_string()@BondError::IllegalMarketAccount,
        constraint=market_account_eth.officer == true@BondError::IllegalMarketAccount,
    )]
    pub market_account_eth: Box<Account<'info, market::Market>>,
    /// CHECK: Verify later
    #[account(
        constraint=market_account_sol.pair == com::FullPositionMarket::SolUsd.to_string()@BondError::IllegalMarketAccount,
        constraint=market_account_sol.officer == true@BondError::IllegalMarketAccount,
    )]
    pub market_account_sol: Box<Account<'info, market::Market>>,
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

pub fn close_position(ctx: Context<ClosePosition>, _identity: u8) -> Result<()> {
    msg!("close position");
    let user_account = &mut ctx.accounts.user_account;
    let market_account = &mut ctx.accounts.market_account;
    let position_account = &mut ctx.accounts.position_account;
    if market_account.status == market::MarketStatus::Frozen {
        return Err(BondError::MarketFrozen.into());
    }
    if position_account.position_status != position::PositionStatus::Normal {
        return Err(BondError::PositionStatusInvalid.into());
    }
    // check user
    let is_user_operator = user_account.authority == ctx.accounts.authority.key();
    let is_robot_operator = com::base_account::get_clearing_robot() == ctx.accounts.authority.key();
    if !is_user_operator && !is_robot_operator {
        return Err(BondError::NoPermission.into());
    }
    let price = market_account.get_price(
        &ctx.accounts.pyth_price_account,
        &ctx.accounts.chianlink_price_account,
    )?;
    // set position data
    if is_user_operator {
        position_account.position_status = position::PositionStatus::NormalClosing;
    }
    if is_robot_operator {
        position_account.position_status = position::PositionStatus::ForceClosing;
    }
    let total_pl = position_account.get_pl_price(&price);
    position_account.profit = total_pl;
    position_account.close_price = match position_account.direction {
        position::Direction::Buy => price.sell_price,
        position::Direction::Sell => price.buy_price,
    };
    position_account.close_real_price = price.real_price;
    position_account.close_spread = price.spread;
    position_account.close_time = Clock::get().unwrap().unix_timestamp;
    position_account.close_operator = ctx.accounts.authority.key();

    let fund_size = position_account.get_fund_size();

    // position settlement
    match position_account.position_type {
        position::PositionType::Full => {}
        position::PositionType::Independent => {}
    }

    let full_level = market_account.vault_full as f64;
    // Priority in settlement from profit and loss pool
    // Whether the basic fund pool is full
    if total_pl >= 0.0 {
        market_account.vault_profit_balance = market_account.vault_profit_balance - total_pl;
        if market_account.vault_profit_balance < 0.0 {
            market_account.vault_base_balance =
                market_account.vault_base_balance + market_account.vault_profit_balance;
            market_account.vault_profit_balance = 0.0;
        }
    } else {
        market_account.vault_base_balance = market_account.vault_base_balance + total_pl.abs();
        let d = market_account.vault_base_balance - full_level;
        if d > 0.0 {
            market_account.vault_profit_balance += d;
            market_account.vault_base_balance = full_level;
        }
    }
    if position_account.position_type == position::PositionType::Independent {
        let mut margin = position_account.margin;
        margin += total_pl;
        if margin > 0.0 {
            user_account.balance += margin;
        } else {
            msg!("The user's initial margin is insufficient to cover the loss");
        }
    } else {
        user_account.balance += total_pl
    }
    match position_account.direction {
        position::Direction::Buy => {
            market_account.long_position_total -= fund_size;
            user_account.position_full_vector -= 1;
        }
        position::Direction::Sell => {
            market_account.short_position_total -= fund_size;
            user_account.position_full_vector -= 1;
        }
    }
    // set user account data
    user_account.margin_total -= position_account.margin;
    match position_account.position_type {
        position::PositionType::Full => {
            user_account.margin_full_total -= position_account.margin;
            match position_account.direction {
                position::Direction::Buy => {
                    user_account.margin_full_buy_total -= position_account.margin
                }
                position::Direction::Sell => {
                    user_account.margin_full_sell_total -= position_account.margin
                }
            }
        }
        position::PositionType::Independent => {
            user_account.margin_independent_total -= position_account.margin;
            match position_account.direction {
                position::Direction::Buy => {
                    user_account.margin_independent_buy_total -= position_account.margin
                }
                position::Direction::Sell => {
                    user_account.margin_independent_sell_total -= position_account.margin
                }
            }
        }
    }
    user_account.update_index_by_close(position_account.position_seed_offset);
    user_account.remove_position_header(position::PositionHeader {
        position_seed_offset: position_account.position_seed_offset,
        open_price: position_account.open_price,
        direction: position_account.direction,
        size: position_account.size,
        margin: position_account.margin,
        market: com::FullPositionMarket::from(market_account.pair.as_str()),
    });
    msg!("close position success!");
    Ok(())
}

#[derive(Accounts)]
pub struct ClosePosition<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    #[account(
        mut,
        // has_one = authority@BondError::UserTransactionAccountMismatch,
        seeds = [com::USER_ACCOUNT_SEED,authority.key().as_ref()],
        bump,
    )]
    pub user_account: Account<'info, user::UserAccount>,
    #[account(
        mut,
        constraint = market_account.key() == position_account.market_account.key()@BondError::AccountNumberNotMatch,
    )]
    pub market_account: Box<Account<'info, market::Market>>,
    #[account(
        mut,
        seeds=[com::POSITION_ACCOUNT_SEED,user_account.authority.key().as_ref(),user_account.key().as_ref(),position_account.position_seed_offset.to_string().as_bytes().as_ref()],
        bump,
    )]
    pub position_account: Account<'info, position::Position>,
    /// CHECK: Verify later
    #[account(
        constraint = market_account.pyth_price_account.key() == pyth_price_account.key()@BondError::InvalidPriceAccount)
    ]
    pub pyth_price_account: AccountInfo<'info>,
    /// CHECK: Verify later
    #[account(
        constraint=market_account.chianlink_price_account.key() == chianlink_price_account.key()@BondError::InvalidPriceAccount)
    ]
    pub chianlink_price_account: AccountInfo<'info>,
}

// get the full position equity
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
