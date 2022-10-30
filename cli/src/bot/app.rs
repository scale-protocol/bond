use crate::com;
use log::*;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::{runtime::Builder, signal};

use super::sub;

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

    let sub = sub::SubClient::new(runtime);
    sub.subscribe_program_accounts(ctx)?;
    let s = sub.runtime.block_on(async { signal::ctrl_c().await });
    match s {
        Ok(()) => {
            info!("got exit signal...Start execution exit.")
        }
        Err(err) => {
            error!("Unable to listen for shutdown signal: {}", err);
        }
    }
    sub.stop();
    Ok(())
}
