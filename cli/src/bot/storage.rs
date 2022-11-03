use crate::com;
use anchor_client::solana_sdk::{account::Account, pubkey::Pubkey};
use sled::{Batch, Db};
use std::fmt;

pub enum Prefix {
    Active = 1,
    History,
}

impl Prefix {
    pub fn get_storage_key(self, pubkey: &Pubkey) -> String {
        format!("{}_{}", self.to_string(), pubkey.to_string())
    }
    pub fn prefix(&self) -> String {
        format!("{}_", self.to_string())
    }
}

impl fmt::Display for Prefix {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let t = match *self {
            Self::Active => "active",
            Self::History => "history",
        };
        write!(f, "{}", t)
    }
}
pub struct Storage {
    db: Db,
}
impl Storage {
    pub fn new(ctx: &com::Context) -> anyhow::Result<Self> {
        let path = ctx.config.store_path.join("accounts");
        let db = sled::open(path).map_err(|e| com::CliError::DBError(e.to_string()))?;
        Ok(Self { db })
    }
    // Active load Active account
    pub fn scan_prefix(&self, p: &Prefix) -> sled::Iter {
        let px = p.prefix();
        self.db.scan_prefix(px.as_bytes())
    }

    fn save_one(&self, pubkey: &Pubkey, account: &Account, p: Prefix) -> anyhow::Result<()> {
        let value = serde_json::to_vec(account)?;
        let key = p.get_storage_key(pubkey);
        self.db.insert(key.as_bytes(), value)?;
        Ok(())
    }
    pub fn save_to_active(&self, pubkey: &Pubkey, account: &Account) -> anyhow::Result<()> {
        self.save_one(pubkey, account, Prefix::Active)
    }
    pub fn save_to_history(&self, pubkey: &Pubkey, account: &Account) -> anyhow::Result<()> {
        self.save_one(pubkey, account, Prefix::History)
    }
    pub fn save_as_history(&self, pubkey: &Pubkey, account: &Account) -> anyhow::Result<()> {
        let p = Prefix::Active;
        let history_p = Prefix::History;
        let value = serde_json::to_vec(account)?;
        let value = value.as_slice();
        let key = p.get_storage_key(pubkey);
        let history_key = history_p.get_storage_key(pubkey);
        self.db
            .transaction::<_, (), anyhow::Error>(|tx| {
                tx.remove(key.as_bytes())?;
                tx.insert(history_key.as_bytes(), value)?;
                Ok(())
            })
            .map_err(|e| com::CliError::DBError(e.to_string()))?;
        Ok(())
    }

    pub fn save_batch(&self, kv: Vec<(&Pubkey, &Account, Prefix)>) -> anyhow::Result<()> {
        let mut batch = Batch::default();
        for v in kv {
            let value = serde_json::to_vec(v.1)?;
            let key = v.2.get_storage_key(v.0);
            batch.insert(key.as_bytes(), value);
        }
        Ok(())
    }
}
