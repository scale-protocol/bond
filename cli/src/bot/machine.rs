use super::storage;
use crate::com;
use anchor_client::anchor_lang::AccountDeserialize;
use anchor_client::solana_sdk::{account::Account, pubkey::Pubkey};
use bond::state::{market, position, user};
use log::{debug, error, info};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::{
    net::TcpStream,
    sync::{mpsc, oneshot},
    task::JoinHandle,
    time::{sleep, Duration},
};
pub enum State {
    Market(market::Market),
    User(user::UserAccount),
    Position(position::Position),
    None,
}

impl<'a> From<&'a Account> for State {
    fn from(account: &'a Account) -> Self {
        match account.data.len() {
            market::Market::LEN => {
                let mut data: &[u8] = &account.data;
                let t = market::Market::try_deserialize(&mut data);
                match t {
                    Ok(r) => Self::Market(r),
                    Err(e) => {
                        error!("deserialize error:{}", e);
                        Self::None
                    }
                }
            }
            user::UserAccount::LEN => {
                let mut data: &[u8] = &account.data;
                let t = user::UserAccount::try_deserialize(&mut data);
                match t {
                    Ok(r) => Self::User(r),
                    Err(e) => {
                        error!("deserialize error:{}", e);
                        Self::None
                    }
                }
            }
            position::Position::LEN => {
                let mut data: &[u8] = &account.data;
                let t = position::Position::try_deserialize(&mut data);
                match t {
                    Ok(r) => Self::Position(r),
                    Err(e) => {
                        error!("deserialize error:{}", e);
                        Self::None
                    }
                }
            }
            _ => Self::None,
        }
    }
}
pub struct StateMap {
    pub market: HashMap<Pubkey, market::Market>,
    pub user: HashMap<Pubkey, user::UserAccount>,
    pub position: HashMap<Pubkey, position::Position>,
    storage: storage::Storage,
}

impl StateMap {
    pub fn new(ctx: com::Context) -> anyhow::Result<Self> {
        let storage = storage::Storage::new(&ctx)?;
        let market: HashMap<Pubkey, market::Market> = HashMap::new();
        let user: HashMap<Pubkey, user::UserAccount> = HashMap::new();
        let position: HashMap<Pubkey, position::Position> = HashMap::new();
        Ok(Self {
            market,
            user,
            position,
            storage,
        })
    }

    pub fn load_active_account_from_local(&mut self) -> anyhow::Result<()> {
        let p = storage::Prefix::Active;
        let px = (&p).prefix();
        let r = self.storage.scan_prefix(&p);
        for i in r {
            match i {
                Ok((k, v)) => {
                    let key = String::from_utf8(k.to_vec())
                        .map_err(|e| com::CliError::JsonError(e.to_string()))?;
                    let pk = &key[px.len()..];
                    debug!("got pubkey from db:{}", pk);
                    let pbk =
                        Pubkey::try_from(pk).map_err(|e| com::CliError::Unknown(e.to_string()))?;
                    let values: Account = serde_json::from_slice(v.to_vec().as_slice())
                        .map_err(|e| com::CliError::JsonError(e.to_string()))?;
                    let s: State = (&values).into();
                    match s {
                        State::Market(m) => {
                            self.market.insert(pbk, m);
                        }
                        State::User(m) => {
                            self.user.insert(pbk, m);
                        }
                        State::Position(m) => {
                            self.position.insert(pbk, m);
                        }
                        State::None => {}
                    }
                }
                Err(e) => {
                    debug!("{}", e);
                }
            }
        }
        Ok(())
    }
}
pub struct Watch {
    account_shutdown_tx: oneshot::Sender<()>,
    price_shutdown_tx: oneshot::Sender<()>,
    pub account_watch_tx: UnboundedSender<(Pubkey, Account)>,
    pub price_watch_tx: UnboundedSender<(Pubkey, Account)>,
    aw: JoinHandle<anyhow::Result<()>>,
    pw: JoinHandle<anyhow::Result<()>>,
}
// type WatchRx=UnboundedSender<(Pubkey, Account);
impl Watch {
    pub async fn new(mp: StateMap) -> Self {
        let (account_watch_tx, account_watch_rx) = mpsc::unbounded_channel::<(Pubkey, Account)>();
        let (account_shutdown_tx, account_shutdown_rx) = oneshot::channel::<()>();
        let (price_watch_tx, price_watch_rx) = mpsc::unbounded_channel::<(Pubkey, Account)>();
        let (price_shutdown_tx, price_shutdown_rx) = oneshot::channel::<()>();
        Self {
            account_shutdown_tx,
            price_shutdown_tx,
            account_watch_tx,
            price_watch_tx,
            aw: tokio::spawn(watch_account(mp, account_watch_rx, account_shutdown_rx)),
            pw: tokio::spawn(watch_price(price_watch_rx, price_shutdown_rx)),
        }
    }
    pub async fn shutdown(self) {
        let _ = self.account_shutdown_tx.send(());
        let _ = self.aw.await;
        let _ = self.price_shutdown_tx.send(());
        let _ = self.pw.await;
    }
}

async fn watch_account(
    mut mp: StateMap,
    mut watch_rx: UnboundedReceiver<(Pubkey, Account)>,
    mut shutdown_rx: oneshot::Receiver<()>,
) -> anyhow::Result<()> {
    info!("start scale program account watch");
    loop {
        tokio::select! {
            _ = (&mut shutdown_rx) => {
                info!("got shutdown signal，break watch account");
                break;
            },
            r = watch_rx.recv()=>{
                match r {
                    Some(rs)=>{
                        let (pubkey,account)=rs;
                        keep_account(&mut mp, pubkey, account);
                    }
                    None=>{}
                }
            }
        }
    }
    Ok(())
}

async fn watch_price(
    mut watch_rx: UnboundedReceiver<(Pubkey, Account)>,
    mut shutdown_rx: oneshot::Receiver<()>,
) -> anyhow::Result<()> {
    info!("start price account watch");
    loop {
        tokio::select! {
            _ = (&mut shutdown_rx) => {
                info!("got shutdown signal，break watch price");
                break;
            },
            r = watch_rx.recv()=>{
                match r {
                    Some(rs)=>{
                        let (pubkey,account)=rs;
                    }
                    None=>{}
                }
            }
        }
    }
    Ok(())
}

fn keep_account(mp: &mut StateMap, pubkey: Pubkey, account: Account) {
    let s: State = (&account).into();
    match s {
        State::Market(m) => {
            if account.lamports <= 0 {
                mp.market.remove(&pubkey);
                save_as_history(mp, pubkey, account);
            } else {
                mp.market.insert(pubkey, m);
                save_to_active(mp, pubkey, account);
            }
        }
        State::User(m) => {
            if account.lamports <= 0 {
                mp.user.remove(&pubkey);
                save_as_history(mp, pubkey, account);
            } else {
                mp.user.insert(pubkey, m);
                save_to_active(mp, pubkey, account);
            }
        }
        State::Position(m) => {
            if account.lamports <= 0
                || m.position_status == position::PositionStatus::NormalClosing
                || m.position_status == position::PositionStatus::ForceClosing
            {
                mp.position.remove(&pubkey);
                save_as_history(mp, pubkey, account);
            } else {
                mp.position.insert(pubkey, m);
                save_to_active(mp, pubkey, account);
            }
        }
        State::None => {}
    }
}
fn save_as_history(mp: &StateMap, pubkey: Pubkey, account: Account) {
    match mp.storage.save_as_history(&pubkey, &account) {
        Ok(()) => {
            debug!("save a account as history success!");
        }
        Err(e) => {
            error!("save a account as history error:{}", e);
        }
    }
}
fn save_to_active(mp: &StateMap, pubkey: Pubkey, account: Account) {
    match mp.storage.save_to_active(&pubkey, &account) {
        Ok(()) => {
            debug!("save a account as active success!");
        }
        Err(e) => {
            error!("save a account as active error:{}", e);
        }
    }
}
pub struct Price {
    pub buy_price: f64,
    pub sell_price: f64,
    pub real_price: f64,
    pub spread: f64,
}
