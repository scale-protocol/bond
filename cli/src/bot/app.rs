use crate::com;
use log::*;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::{runtime::Builder, signal};

use super::{machine, sub};

pub fn run(ctx: com::Context) -> anyhow::Result<()> {
    let threads: usize = 1;
    let runtime = Builder::new_multi_thread()
        .worker_threads(threads)
        .thread_name_fn(|| {
            static ATOMIC_ID: AtomicUsize = AtomicUsize::new(0);
            let id = ATOMIC_ID.fetch_add(1, Ordering::Relaxed);
            format!("scale-robot-{}", id)
        })
        .enable_all()
        .build()
        .map_err(|e| com::CliError::TokioRuntimeCreateField(e.to_string()))?;
    let mut sate_map = machine::StateMap::new(ctx.config.clone())?;

    sate_map.load_active_account_from_local()?;

    let config = ctx.config.clone();

    let task = runtime.spawn(async move {
        let watch = machine::Watch::new(sate_map).await;
        let sub = sub::SubAccount::new(
            config,
            watch.account_watch_tx.clone(),
            watch.price_watch_tx.clone(),
        )
        .await;
        (watch, sub)
    });

    let s = runtime.block_on(async { signal::ctrl_c().await });
    match s {
        Ok(()) => {
            info!("got exit signal...Start execution exit.")
        }
        Err(err) => {
            error!("Unable to listen for shutdown signal: {}", err);
        }
    }
    runtime.block_on(async {
        let (wt, sb) = task.await.unwrap();
        wt.shutdown().await;
        sb.shutdown().await;
        info!("robot server shutdown!");
    });
    Ok(())
}
