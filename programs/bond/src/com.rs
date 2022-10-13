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

/// Mint address of the vault token, the test version is the circulating token issued by the project,
/// and the official network is USDC
pub const VAULT_MINT: &str = "6fsNvxjjePiMGeHUFKUMhjG9CGAVRVwwQjaUJPrLYUu3";

/// Address of project development fund wallet
pub const PROJECT_FUND_WALLET: &str = "Bu91vdLYSmiip8fS7ijzTcFAnu3TNCUA7kfj2pRMzC9T";

/// Insurance fund address
pub const INSURANCE_FUND_ADDRESS: &str = "Bu91vdLYSmiip8fS7ijzTcFAnu3TNCUA7kfj2pRMzC9T";

#[cfg(feature = "mainnetbeta")]
pub fn get_vault_mint() -> Pubkey {
    mint::USDC
}
#[cfg(feature = "testnet")]
pub fn get_vault_mint() -> Pubkey {
    Pubkey::from_str(VAULT_MINT).unwrap()
}
#[cfg(any(feature = "devnet", feature = "localhost"))]
pub fn get_vault_mint() -> Pubkey {
    Pubkey::from_str(VAULT_MINT).unwrap()
}

pub fn get_project_fund_wallet() -> Pubkey {
    Pubkey::from_str(PROJECT_FUND_WALLET).unwrap()
}

pub fn get_insurance_fund_address() -> Pubkey {
    Pubkey::from_str(INSURANCE_FUND_ADDRESS).unwrap()
}

pub const VAULT_TOKEN_ACCOUNT_SEED: &[u8] = b"scale_vault";

pub const VAULT_TOKEN_AUTHORITY_SEED: &[u8] = b"scale_vault_authority";

pub const USER_ACCOUNT_SEED: &[u8] = b"scale_user_account";

pub const MARKET_ACCOUNT_SEED: &[u8] = b"scale_market_account";

pub const POSITION_INDEX_ACCOUNT_SEED: &[u8] = b"scale_position_index_account";

pub const POSITION_ACCOUNT_SEED: &[u8] = b"scale_position_account";
