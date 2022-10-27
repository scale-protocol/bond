use crate::com;
use anchor_client::{self, Cluster};
use anyhow::Ok;
use home;
use log::debug;
use std::{fs, path::PathBuf, str::FromStr};
extern crate serde;
extern crate serde_yaml;

use serde::{Deserialize, Serialize};
#[derive(Debug, Clone)]
pub struct Config {
    pub config_file: PathBuf,
    pub cluster: anchor_client::Cluster,
    pub wallet: PathBuf,
    pub store_path: PathBuf,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigBody {
    pub rpc_url: String,
    pub ws_url: String,
    pub keypair_path: String,
    pub cluster: String,
    pub store_path: String,
}

impl From<&Config> for ConfigBody {
    fn from(c: &Config) -> Self {
        Self {
            rpc_url: c.cluster.url().to_string(),
            ws_url: c.cluster.ws_url().to_string(),
            keypair_path: c.wallet.to_str().unwrap().to_string(),
            cluster: c.cluster.to_string(),
            store_path: c.store_path.to_str().unwrap().to_string(),
        }
    }
}

impl From<&ConfigBody> for Config {
    fn from(c: &ConfigBody) -> Self {
        let config = Config::default();
        Self {
            config_file: config.config_file,
            cluster: match c.cluster.as_str() {
                "debug" => Cluster::Debug,
                "devnet" => Cluster::Devnet,
                "localnet" => Cluster::Localnet,
                "testnet" => Cluster::Testnet,
                "mainnet" => Cluster::Mainnet,
                _ => Cluster::Custom(c.rpc_url.clone(), c.ws_url.clone()),
            },
            wallet: PathBuf::from(c.keypair_path.clone()),
            store_path: PathBuf::from(c.store_path.clone()),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        let home_dir = match home::home_dir() {
            Some(p) => p,
            None => PathBuf::from("/tmp/"),
        };
        let home_dir = home_dir.join(".scale");
        if !home_dir.is_dir() {
            fs::create_dir(&home_dir).unwrap();
        }
        Config {
            config_file: home_dir.join("config.yml"),
            cluster: Cluster::Localnet,
            wallet: home_dir.join("id.json"),
            store_path: home_dir.join("params"),
        }
    }
}
impl Config {
    pub fn init(&self) {
        let config_body: ConfigBody = self.into();
        // save
        debug!("init config file:{:?}", self.config_file);
        fs::write(
            &self.config_file.clone(),
            serde_yaml::to_string(&config_body).unwrap().as_bytes(),
        )
        .unwrap()
    }
    pub fn get(&self) {
        println!("Config File : {:?}\nCluster : {}\nWallet keypair file : {:?}\nLocal store path : {:?}\nRpc url : {}\nWs url : {}", 
        self.config_file,
        self.cluster,
        self.wallet,
        self.store_path,
        self.cluster.url(),
        self.cluster.ws_url()
    );
    }
    pub fn set(
        &mut self,
        store_path: Option<&PathBuf>,
        keypair_file: Option<&PathBuf>,
        rpc_url: Option<&String>,
        ws_url: Option<&String>,
        cluster: Option<&String>,
    ) {
        match store_path {
            Some(s) => self.store_path = s.to_path_buf(),
            None => {}
        }
        match keypair_file {
            Some(k) => self.wallet = k.to_path_buf(),
            None => {}
        }
        match cluster {
            Some(c) => self.cluster = Cluster::from_str(c.as_str()).unwrap(),
            None => {}
        }
        match rpc_url {
            Some(r) => {
                self.cluster = Cluster::Custom(r.to_string(), self.cluster.ws_url().to_string());
            }
            None => {}
        }
        match ws_url {
            Some(r) => {
                self.cluster = Cluster::Custom(self.cluster.url().to_string(), r.to_string());
            }
            None => {}
        }
        let config = &(*self);
        let config_body: ConfigBody = config.into();
        // save
        debug!("init config file:{:?}", self.config_file);
        fs::write(
            &self.config_file.clone(),
            serde_yaml::to_string(&config_body).unwrap().as_bytes(),
        )
        .unwrap();
        self.get();
    }
    // load config from local file
    pub fn load(&mut self) -> anyhow::Result<()> {
        let config_body: ConfigBody = serde_yaml::from_str(
            fs::read_to_string(&self.config_file)
                .map_err(|e| com::CliError::LoadConfigFileError(e.to_string()))?
                .as_str(),
        )
        .map_err(|e| com::CliError::LoadConfigFileError(e.to_string()))?;
        let s: Config = (&config_body).into();
        self.config_file = s.config_file;
        self.cluster = s.cluster;
        self.store_path = s.store_path;
        self.wallet = s.wallet;
        Ok(())
    }
}
