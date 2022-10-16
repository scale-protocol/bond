use std::str::FromStr;

use anchor_lang::prelude::Pubkey;
use anchor_spl::mint;

/// The exposure ratio should not exceed 70% of the current pool,
/// so as to avoid the risk that the platform's current pool is empty.
pub const POSITION_DIFF_PROPORTION: f32 = 0.7;
/// The proportion of unidirectional positions shall not exceed 150% of the flow pool,
/// so as to avoid the risk of malicious position opening.
pub const POSITION_PROPORTION: f32 = 1.5;
/// Funding rate, which is 1% of the proportion of exposed funds in the liquidity pool.
///  For example, when the exposure proportion is 70%, this value is 7/1000.
pub const FUND_RATE: f32 = 0.01;
/// The liquidation line ratio means that if the user's margin loss exceeds this ratio in one quotation,
/// the system will be liquidated and the position will be forced to close.
pub const BURST_RATE: f32 = 0.5;
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
    fn get_project_fund_wallet() -> Pubkey {
        Pubkey::try_from("Bu91vdLYSmiip8fS7ijzTcFAnu3TNCUA7kfj2pRMzC9T").unwrap()
    }
    /// Insurance fund address
    fn get_insurance_fund_wallet() -> Pubkey {
        Pubkey::try_from("Bu91vdLYSmiip8fS7ijzTcFAnu3TNCUA7kfj2pRMzC9T").unwrap()
    }
    /// The team authorized account is used to initialize and set the official trading market
    fn get_team_authority() -> Pubkey {
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
    fn get_project_fund_wallet() -> Pubkey {
        Pubkey::try_from("Bu91vdLYSmiip8fS7ijzTcFAnu3TNCUA7kfj2pRMzC9T").unwrap()
    }
    /// Insurance fund address
    fn get_insurance_fund_wallet() -> Pubkey {
        Pubkey::try_from("Bu91vdLYSmiip8fS7ijzTcFAnu3TNCUA7kfj2pRMzC9T").unwrap()
    }
    /// The team authorized account is used to initialize and set the official trading market
    fn get_team_authority() -> Pubkey {
        Pubkey::try_from("9Q2SWBAXzrFeYu2k8diw1Q9bMD7Qo2pLMyjXU4iLT5iM").unwrap()
    }
}
#[cfg(any(feature = "devnet", feature = "localhost"))]
pub mod base_account {
    use super::*;
    /// Mint address of the vault token, the test version is the circulating token issued by the project,
    /// and the official network is USDC
    pub fn get_vault_mint() -> Pubkey {
        Pubkey::try_from("6fsNvxjjePiMGeHUFKUMhjG9CGAVRVwwQjaUJPrLYUu3").unwrap()
    }
    /// Address of project development fund wallet
    fn get_project_fund_wallet() -> Pubkey {
        Pubkey::try_from("Bu91vdLYSmiip8fS7ijzTcFAnu3TNCUA7kfj2pRMzC9T").unwrap()
    }
    /// Insurance fund address
    fn get_insurance_fund_wallet() -> Pubkey {
        Pubkey::try_from("Bu91vdLYSmiip8fS7ijzTcFAnu3TNCUA7kfj2pRMzC9T").unwrap()
    }
    /// The team authorized account is used to initialize and set the official trading market
    fn get_team_authority() -> Pubkey {
        Pubkey::try_from("9Q2SWBAXzrFeYu2k8diw1Q9bMD7Qo2pLMyjXU4iLT5iM").unwrap()
    }
}

pub enum OfficialMarket {}
