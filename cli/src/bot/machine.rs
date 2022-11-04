use super::{price, storage};
use crate::{com, config};
use anchor_client::anchor_lang::AccountDeserialize;
use anchor_client::solana_sdk::{account::Account, pubkey::Pubkey};
use bond::state::{market, position, user};
use dashmap::DashMap;
use log::{debug, error, info};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::{
    sync::{mpsc, oneshot},
    task::JoinHandle,
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

type DmMarket = DashMap<Pubkey, market::Market>;
type DmUser = DashMap<Pubkey, user::UserAccount>;
type DmPosition = DashMap<Pubkey, position::Position>;
type DmPrice = DashMap<Pubkey, market::Price>;
// key is price account,value is market account
type DmIdxPriceMarket = DashMap<Pubkey, Pubkey>;

#[derive(Clone)]
pub struct StateMap {
    pub market: DmMarket,
    pub user: DmUser,
    pub position: DmPosition,
    pub price_account: DmPrice,
    pub price_idx_price_account: DmIdxPriceMarket,
    storage: storage::Storage,
}

impl StateMap {
    pub fn new(config: config::Config) -> anyhow::Result<Self> {
        let storage = storage::Storage::new(config)?;
        let market: DmMarket = DashMap::new();
        let user: DmUser = DashMap::new();
        let position: DmPosition = DashMap::new();
        let price_account: DmPrice = DashMap::new();
        let price_idx_price_account: DmIdxPriceMarket = DashMap::new();
        Ok(Self {
            market,
            user,
            position,
            storage,
            price_account,
            price_idx_price_account,
        })
    }

    pub fn load_active_account_from_local(&mut self) -> anyhow::Result<()> {
        info!("start load active account from local!");
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
        info!("complete load active account from local!");
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
            aw: tokio::spawn(watch_account(
                mp.clone(),
                account_watch_rx,
                account_shutdown_rx,
            )),
            pw: tokio::spawn(watch_price(mp.clone(), price_watch_rx, price_shutdown_rx)),
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
    info!("start scale program account watch ...");
    loop {
        tokio::select! {
            _ = (&mut shutdown_rx) => {
                info!("got shutdown signal，break watch account");
                break;
            },
            r = watch_rx.recv()=>{
                match r {
                    Some(rs)=>{
                        let (pubkey,account) = rs;
                        debug!("account channel got data : {:?},{:?}",pubkey,account);
                        keep_account(&mut mp, pubkey, account);
                    }
                    None=>{
                        debug!("account channel got none : {:?}",r);
                    }
                }
            }
        }
    }
    Ok(())
}

async fn watch_price(
    mut mp: StateMap,
    mut watch_rx: UnboundedReceiver<(Pubkey, Account)>,
    mut shutdown_rx: oneshot::Receiver<()>,
) -> anyhow::Result<()> {
    info!("start price account watch...");
    loop {
        tokio::select! {
            _ = (&mut shutdown_rx) => {
                info!("got shutdown signal，break watch price");
                break;
            },
            r = watch_rx.recv()=>{
                match r {
                    Some(rs)=>{
                        let (pubkey,account) = rs;
                        keep_price(&mut mp, pubkey, account);
                    }
                    None=>{}
                }
            }
        }
    }
    Ok(())
}

fn keep_price(mp: &mut StateMap, pubkey: Pubkey, mut account: Account) {
    match mp.price_idx_price_account.get(&pubkey) {
        Some(k) => match mp.market.get(&k) {
            Some(m) => match price::get_price_from_pyth(&pubkey, &mut account) {
                Ok(p) => {
                    let spread = com::f64_round(p * m.spread);
                    let price = market::Price {
                        buy_price: com::f64_round(p + spread),
                        sell_price: com::f64_round(p - spread),
                        real_price: p,
                        spread,
                    };
                    mp.price_account.insert(pubkey, price);
                }
                Err(e) => {
                    error!("{}", e);
                }
            },
            None => {
                error!("keep price error,get index but cannot get market data");
            }
        },
        None => {
            debug!(
                "Can not found market of price account,ignore it .{}",
                pubkey
            );
        }
    }
}
fn keep_account(mp: &mut StateMap, pubkey: Pubkey, account: Account) {
    let s: State = (&account).into();
    match s {
        State::Market(m) => {
            let pyth_account = m.pyth_price_account;
            let chainlink_account = m.chianlink_price_account;
            if account.lamports <= 0 {
                mp.market.remove(&pubkey);
                mp.price_idx_price_account.remove(&pyth_account);
                mp.price_idx_price_account.remove(&chainlink_account);
                save_as_history(mp, pubkey, account);
            } else {
                mp.market.insert(pubkey, m);
                mp.price_idx_price_account.insert(pyth_account, pubkey);
                mp.price_idx_price_account.insert(chainlink_account, pubkey);
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
        State::None => {
            error!(
                "Unrecognized structure of account:{:?},{:?}",
                pubkey, account
            );
        }
    }
}

fn save_as_history(mp: &StateMap, pubkey: Pubkey, account: Account) {
    match mp.storage.save_as_history(&pubkey, &account) {
        Ok(()) => {
            debug!("save a account as history success!,account:{}", pubkey);
        }
        Err(e) => {
            error!("save a account as history error:{},account:{}", e, pubkey);
        }
    }
}

fn save_to_active(mp: &StateMap, pubkey: Pubkey, account: Account) {
    match mp.storage.save_to_active(&pubkey, &account) {
        Ok(()) => {
            debug!("save a account as active success!,account:{}", pubkey);
        }
        Err(e) => {
            error!("save a account as active error:{},account:{}", e, pubkey);
        }
    }
}
