use anchor_lang::prelude::*;
use anchor_spl::mint;
use std::convert::TryFrom;
use std::fmt;

/// The exposure ratio should not exceed 70% of the current pool,
/// so as to avoid the risk that the platform's current pool is empty.
pub const POSITION_DIFF_PROPORTION: f64 = 0.7;
/// The proportion of unidirectional positions shall not exceed 150% of the flow pool,
/// so as to avoid the risk of malicious position opening.
pub const POSITION_PROPORTION: f64 = 1.5;
/// The size of a single position shall not be greater than 20% of the exposure
pub const POSITION_PROPORTION_ONE: f64 = 0.2;
/// Funding rate, which is 1% of the proportion of exposed funds in the liquidity pool.
///  For example, when the exposure proportion is 70%, this value is 7/1000.
pub const FUND_RATE: f64 = 0.01;
/// The liquidation line ratio means that if the user's margin loss exceeds this ratio in one quotation,
/// the system will be liquidated and the position will be forced to close.
pub const BURST_RATE: f64 = 0.5;
pub const MAX_LEVERAGE: u16 = 125;

pub const VAULT_TOKEN_ACCOUNT_SEED: &[u8] = b"scale_vault";
pub const VAULT_TOKEN_AUTHORITY_SEED: &[u8] = b"scale_vault_authority";
pub const USER_ACCOUNT_SEED: &[u8] = b"scale_user_account";
pub const MARKET_ACCOUNT_SEED: &[u8] = b"scale_market_account";
pub const POSITION_ACCOUNT_SEED: &[u8] = b"scale_position_account";

#[cfg(feature = "mainnetbeta")]
pub mod base_account {
    use super::*;
    /// Mint address of the vault token, the test version is the circulating token issued by the project,
    /// and the official network is USDC
    pub fn get_vault_mint() -> Pubkey {
        mint::USDC
    }

    /// Address of project development fund wallet
    pub fn get_project_fund_wallet() -> Pubkey {
        Pubkey::try_from("Bu91vdLYSmiip8fS7ijzTcFAnu3TNCUA7kfj2pRMzC9T").unwrap()
    }
    /// Insurance fund address
    pub fn get_insurance_fund_wallet() -> Pubkey {
        Pubkey::try_from("Bu91vdLYSmiip8fS7ijzTcFAnu3TNCUA7kfj2pRMzC9T").unwrap()
    }
    /// The team authorized account is used to initialize and set the official trading market
    pub fn get_team_authority() -> Pubkey {
        Pubkey::try_from("9Q2SWBAXzrFeYu2k8diw1Q9bMD7Qo2pLMyjXU4iLT5iM").unwrap()
    }

    pub fn get_pyth_price_account_btc() -> Pubkey {
        Pubkey::try_from("GVXRSBjFk6e6J3NbVPXohDJetcTjaeeuykUpbQF8UoMU").unwrap()
    }
    pub fn get_pyth_price_account_eth() -> Pubkey {
        Pubkey::try_from("JBu1AL4obBcCMqKBBxhpWCNUt136ijcuMZLFvTP7iWdB").unwrap()
    }
    pub fn get_pyth_price_account_sol() -> Pubkey {
        Pubkey::try_from("H6ARHf6YXhGYeQfUzQNGk6rDNnLBQKrenN712K4AQJEG").unwrap()
    }
    pub fn get_chainlink_price_account_btc() -> Pubkey {
        Pubkey::try_from("CGmWwBNsTRDENT5gmVZzRu38GnNnMm1K5C3sFiUUyYQX").unwrap()
    }
    pub fn get_chainlink_price_account_eth() -> Pubkey {
        Pubkey::try_from("5WyTBrEgvkAXjTdYTLY9PVrztjmz4edP5W9wks9KPFg5").unwrap()
    }
    pub fn get_chainlink_price_account_sol() -> Pubkey {
        Pubkey::try_from("CcPVS9bqyXbD9cLnTbhhHazLsrua8QMFUHTutPtjyDzq").unwrap()
    }
    pub fn get_clearing_robot() -> Pubkey {
        Pubkey::try_from("9Q2SWBAXzrFeYu2k8diw1Q9bMD7Qo2pLMyjXU4iLT5iM").unwrap()
    }
}
#[cfg(feature = "testnet")]
pub mod base_account {
    use super::*;
    /// Mint address of the vault token, the test version is the circulating token issued by the project,
    /// and the official network is USDC
    pub fn get_vault_mint() -> Pubkey {
        Pubkey::try_from("6fsNvxjjePiMGeHUFKUMhjG9CGAVRVwwQjaUJPrLYUu3").unwrap()
    }
    /// Address of project development fund wallet
    pub fn get_project_fund_wallet() -> Pubkey {
        Pubkey::try_from("Bu91vdLYSmiip8fS7ijzTcFAnu3TNCUA7kfj2pRMzC9T").unwrap()
    }
    /// Insurance fund address
    pub fn get_insurance_fund_wallet() -> Pubkey {
        Pubkey::try_from("Bu91vdLYSmiip8fS7ijzTcFAnu3TNCUA7kfj2pRMzC9T").unwrap()
    }
    /// The team authorized account is used to initialize and set the official trading market
    pub fn get_team_authority() -> Pubkey {
        Pubkey::try_from("9Q2SWBAXzrFeYu2k8diw1Q9bMD7Qo2pLMyjXU4iLT5iM").unwrap()
    }
    pub fn get_pyth_price_account_btc() -> Pubkey {
        Pubkey::try_from("DJW6f4ZVqCnpYNN9rNuzqUcCvkVtBgixo8mq9FKSsCbJ").unwrap()
    }
    pub fn get_pyth_price_account_eth() -> Pubkey {
        Pubkey::try_from("7A98y76fcETLHnkCxjmnUrsuNrbUae7asy4TiVeGqLSs").unwrap()
    }
    pub fn get_pyth_price_account_sol() -> Pubkey {
        Pubkey::try_from("7VJsBtJzgTftYzEeooSDYyjKXvYRWJHdwvbwfBvTg9K").unwrap()
    }
    pub fn get_chainlink_price_account_btc() -> Pubkey {
        Pubkey::try_from("").unwrap()
    }
    pub fn get_chainlink_price_account_eth() -> Pubkey {
        Pubkey::try_from("").unwrap()
    }
    pub fn get_chainlink_price_account_sol() -> Pubkey {
        Pubkey::try_from("").unwrap()
    }
    pub fn get_clearing_robot() -> Pubkey {
        Pubkey::try_from("9Q2SWBAXzrFeYu2k8diw1Q9bMD7Qo2pLMyjXU4iLT5iM").unwrap()
    }
}
#[cfg(any(feature = "devnet", feature = "localhost"))]
pub mod base_account {
    use super::*;
    /// Mint address of the vault token, the test version is the circulating token issued by the project,
    /// and the official network is USDC
    pub fn get_vault_mint() -> Pubkey {
        Pubkey::try_from("BKCWDCwVmcS6hP9K7NnFk4UQXJGLDqcBNeqzJ7jJ5xNV").unwrap()
    }
    /// Address of project development fund wallet
    pub fn get_project_fund_wallet() -> Pubkey {
        Pubkey::try_from("Bu91vdLYSmiip8fS7ijzTcFAnu3TNCUA7kfj2pRMzC9T").unwrap()
    }
    /// Insurance fund address
    pub fn get_insurance_fund_wallet() -> Pubkey {
        Pubkey::try_from("Bu91vdLYSmiip8fS7ijzTcFAnu3TNCUA7kfj2pRMzC9T").unwrap()
    }
    /// The team authorized account is used to initialize and set the official trading market
    pub fn get_team_authority() -> Pubkey {
        Pubkey::try_from("9Q2SWBAXzrFeYu2k8diw1Q9bMD7Qo2pLMyjXU4iLT5iM").unwrap()
    }
    pub fn get_pyth_price_account_btc() -> Pubkey {
        Pubkey::try_from("HovQMDrbAgAYPCmHVSrezcSmkMtXSSUsLDFANExrZh2J").unwrap()
    }
    pub fn get_pyth_price_account_eth() -> Pubkey {
        Pubkey::try_from("EdVCmQ9FSPcVe5YySXDPCRmc8aDQLKJ9xvYBMZPie1Vw").unwrap()
    }
    pub fn get_pyth_price_account_sol() -> Pubkey {
        Pubkey::try_from("J83w4HKfqxwcq3BEMMkPFSppX3gqekLyLJBexebFVkix").unwrap()
    }
    pub fn get_chainlink_price_account_btc() -> Pubkey {
        Pubkey::try_from("CzZQBrJCLqjXRfMjRN3fhbxur2QYHUzkpaRwkWsiPqbz").unwrap()
    }
    pub fn get_chainlink_price_account_eth() -> Pubkey {
        Pubkey::try_from("2ypeVyYnZaW2TNYXXTaZq9YhYvnqcjCiifW1C6n8b7Go").unwrap()
    }
    pub fn get_chainlink_price_account_sol() -> Pubkey {
        Pubkey::try_from("HgTtcbcmp5BeThax5AU8vg4VwK79qAvAKKFMs8txMLW6").unwrap()
    }
    pub fn get_clearing_robot() -> Pubkey {
        Pubkey::try_from("9Q2SWBAXzrFeYu2k8diw1Q9bMD7Qo2pLMyjXU4iLT5iM").unwrap()
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum FullPositionMarket {
    BtcUsd = 1,
    EthUsd,
    SolUsd,
    None,
}

impl<'a> From<&'a str> for FullPositionMarket {
    fn from(value: &'a str) -> Self {
        let c = value.as_bytes();
        match c {
            b"BTC/USD" => Self::BtcUsd,
            b"ETH/USD" => Self::EthUsd,
            b"SOL/USD" => Self::SolUsd,
            _ => Self::None,
        }
    }
}
impl fmt::Display for FullPositionMarket {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let t = match *self {
            Self::BtcUsd => "BTC/USD",
            Self::EthUsd => "ETH/USD",
            Self::SolUsd => "SOL/USD",
            Self::None => "None",
        };
        write!(f, "{}", t)
    }
}

pub fn f64_round(f: f64) -> f64 {
    (f * 100.0).round() / 100.0
}
