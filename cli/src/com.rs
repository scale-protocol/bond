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
    #[error("Subscription account failed: {0}")]
    SubscriptionAccountFailed(String),
    #[error("Can not create tokio runtime: {0}")]
    TokioRuntimeCreateField(String),
    #[error("Can not create local db:{0}")]
    DBError(String),
    #[error("Error in json parsing:{0}")]
    JsonError(String),
    #[error("deserialize error:{0}")]
    DeserializeError(String),
    #[error("get price error{0}")]
    PriceError(String),
}
pub fn id() -> Pubkey {
    Pubkey::try_from("3CuC9qc7ehNu3MrGrqDMu6it2g71dFJTKn7184sb1TuJ").unwrap()
}
pub struct Context<'a> {
    pub config: &'a config::Config,
    pub client: &'a anchor_client::Client,
}

impl<'a> Context<'a> {
    pub fn new(config: &'a config::Config, client: &'a anchor_client::Client) -> Self {
        Self { config, client }
    }

    pub fn new_client(c: &'a config::Config) -> anchor_client::Client {
        let payer = read_keypair_file(&c.wallet).expect("Cant not init wallet keypair");
        anchor_client::Client::new_with_options(
            c.cluster.clone(),
            Rc::new(payer),
            CommitmentConfig::processed(),
        )
    }
}
pub fn f64_round(f: f64) -> f64 {
    (f * 100.0).round() / 100.0
}
