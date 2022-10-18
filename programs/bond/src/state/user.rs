use std::i16::MAX;

use crate::com;
use crate::errors::BondError;
use crate::state::position;
use anchor_lang::{accounts, prelude::*};
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};
#[account]
pub struct UserAccount {
    /// Account owner wallet address
    pub authority: Pubkey,
    /// The position offset.
    /// This value is increased by one each time the position is opened to determine
    ///  the seeds index number of the position (this value can be used as the order number).
    pub position_seed_offset: u32,
    /// Balance of user account (maintain the deposit,
    ///  and the balance here will be deducted when the deposit used in the full position mode is deducted)
    pub balance: f64,
    /// User settled profit
    pub profit: f64,
    /// Total amount of margin used.
    pub margin_total: f64,
    /// Total amount of used margin in full warehouse mode.
    pub margin_full_total: f64,
    /// Total amount of used margin in independent position mode.
    pub margin_independent_total: f64,
    /// space for future derived values
    pub drv1: u8,
    /// space for future derived values
    pub drv2: u16,
    /// space for future derived values
    pub drv3: u32,
    /// space for future derived values
    pub drv4: u64,
    /// Open order offset set
    pub open_position_index: Vec<u32>,
    /// Closed order offset set
    pub close_position_index: Vec<u32>,
    /// The position header being opened, which is used to calculate the account net value
    pub open_position_headers: Vec<position::PositionHeader>,
}

/// You can only keep so many order indexes at most.
/// To view all orders, you need to traverse from the beginning
/// We are still determining the range of this value depending on the node calculation force and use cost
const MAX_INDEX_SIZE: usize = 10000;
/// Number of full warehouses allowed to be opened
/// We are still determining the range of this value depending on the node calculation force and use cost
const MAX_OPEN_FULL_POSITION_SET_SIZE: usize = 10000;

impl UserAccount {
    pub const LEN: usize = 32
        + 4
        + 8 * 5
        + (1 + 2 + 4 + 8)
        + (4 + 4 * MAX_INDEX_SIZE) * 2
        + (4 + position::PositionHeader::LEN * MAX_OPEN_FULL_POSITION_SET_SIZE);

    pub fn update_index_by_close(&mut self, offset: u32) {
        if offset <= 0 {
            return;
        }
        // delete the offset item from open list
        // and add the offset item to close list
        self.open_position_index.retain(|&x| x != offset);
        self.close_position_index.push(offset);
        if self.close_position_index.len() > MAX_INDEX_SIZE {
            self.close_position_index.remove(0);
        }
    }
    pub fn update_index_by_open(&mut self, offset: u32) {
        if offset <= 0 {
            return;
        }
        self.open_position_index.push(offset);
        if self.open_position_index.len() > MAX_INDEX_SIZE {
            self.close_position_index.remove(0);
        }
    }
}
