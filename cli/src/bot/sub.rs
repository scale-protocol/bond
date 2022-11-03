use {
    crate::com,
    anchor_client::solana_sdk::account::Account,
    anchor_client::solana_sdk::commitment_config::CommitmentConfig,
    log::{debug, error, info},
    solana_account_decoder::UiAccountEncoding,
    solana_client::nonblocking::{pubsub_client, rpc_client},
    solana_client::rpc_config::{RpcAccountInfoConfig, RpcProgramAccountsConfig},
    tokio::{self, sync::watch},
    tokio_stream::StreamExt,
};

pub struct SubClient {
    pub runtime: tokio::runtime::Runtime,
    close_tx: watch::Sender<bool>,
}

impl SubClient {
    pub fn new(runtime: tokio::runtime::Runtime) -> Self {
        let (close_tx, _) = watch::channel(false);
        Self { runtime, close_tx }
    }

    pub fn subscribe_program_accounts(&self, ctx: com::Context) -> anyhow::Result<()> {
        let config = ctx.config.clone();
        let mut close_rx = self.close_tx.subscribe();

        self.runtime.spawn(async move {
            let sol_sub_client = pubsub_client::PubsubClient::new(config.cluster.ws_url())
                .await
                .map_err(|e| {
                    debug!("{:#?}", e);
                    com::CliError::SubscriptionAccountFailed(e.to_string())
                })
                .unwrap();
            info!("start subscription ...");

            let rpc_config = RpcProgramAccountsConfig {
                filters: None,
                account_config: RpcAccountInfoConfig {
                    encoding: Some(UiAccountEncoding::Base64Zstd),
                    commitment: Some(CommitmentConfig::finalized()),
                    data_slice: None,
                    min_context_slot: None,
                },
                with_context: None,
            };

            let id = com::id();
            let (mut s, _r) = sol_sub_client
                .program_subscribe(&id, Some(rpc_config))
                .await
                .unwrap();
            let mut s = s.as_mut();

            loop {
                tokio::select! {
                    response = s.next() => {
                        match response {
                            Some(i_account)=>{
                                let pda_pubkey = i_account.value.pubkey;
                                let pda_account:Option<Account> = i_account.value.account.decode();
                                match pda_account {
                                    Some(account)=>{
                                        debug!("got account: {:?} data: {:#?},len:{}",pda_pubkey,account,account.data.len());
                                    }
                                    None=>{
                                        error!("Can not decode account,got None");
                                    }
                                }
                            }
                            None=>{
                                info!("message channel close");
                                break;
                            }
                        }
                    }
                    close = close_rx.changed() => {
                        if close.is_ok(){
                            info!("got close message,sub task exit.");
                            break;
                        }
                        if close.is_err(){
                            info!("the watch channel is close,sub task exit.");
                            break;
                        }
                    },
                }
            }
        });
        Ok(())
    }

    pub fn stop(&self) {
        if let Err(err) = self.close_tx.send(true) {
            error!("can'n send close message:{:?}", err);
        }
    }

    pub fn get_all_program_accounts(&self, ctx: com::Context) {
        let client = rpc_client::RpcClient::new(ctx.config.cluster.url().to_string());
        let id = com::id();
        self.runtime.spawn(async move {
            let accounts = client.get_program_accounts(&id);
        });
    }
}
