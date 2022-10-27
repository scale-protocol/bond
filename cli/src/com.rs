use std::rc::Rc;

use anchor_client::solana_sdk::pubkey::Pubkey;
use thiserror::Error;

use crate::config;
use anchor_client::solana_sdk::commitment_config::CommitmentConfig;
use anchor_client::solana_sdk::signature::read_keypair_file;
#[derive(Error, Debug)]
pub enum CliError {
    #[error("Unknown error: {0}")]
    Unknown(String),
    #[error("Can not load config file from local:{0}")]
    LoadConfigFileError(String),
}
pub fn id() -> Pubkey {
    Pubkey::try_from("3CuC9qc7ehNu3MrGrqDMu6it2g71dFJTKn7184sb1TuJ").unwrap()
}
pub struct Context<'a> {
    pub config: &'a config::Config,
    pub client: anchor_client::Client,
}

impl<'a> Context<'a> {
    pub fn new(c: &'a config::Config) -> Self {
        let payer = read_keypair_file(&c.wallet).expect("Cant not init wallet keypair");
        Self {
            config: c,
            client: anchor_client::Client::new_with_options(
                c.cluster.clone(),
                Rc::new(payer),
                CommitmentConfig::processed(),
            ),
        }
    }
}
