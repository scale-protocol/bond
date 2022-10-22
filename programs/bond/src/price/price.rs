use crate::com;
use crate::errors::BondError;
use anchor_lang::prelude::*;
use pyth_sdk_solana::{load_price_feed_from_account_info, Price, PriceFeed};

pub fn get_price(price_account_info: &AccountInfo) -> Result<f64> {
    get_price_from_pyth(price_account_info)
    // todo ,if error then get price from chainlink
}

// get price from pyth.network
#[cfg(any(feature = "devnet", feature = "testnet", feature = "mainnetbeta"))]
fn get_price_from_pyth(price_account_info: &AccountInfo) -> Result<f64> {
    let price_feed: PriceFeed =
        load_price_feed_from_account_info(&price_account_info).map_err(|err| {
            msg!("load_price_feed_from_account_info error:{:?}", err);
            BondError::GetPriceFailedFromPyth
        })?;
    let current_price: Price = price_feed
        .get_current_price()
        .ok_or(BondError::GetPriceFailedFromPyth)?;
    let price = com::f64_round(
        current_price.price as f64 / 10u64.pow(current_price.expo.abs() as u32) as f64,
    );
    Ok(price)
}

#[cfg(any(feature = "localhost"))]
fn get_price_from_pyth(_price_account_info: &AccountInfo) -> Result<f64> {
    Ok(1.26)
}
