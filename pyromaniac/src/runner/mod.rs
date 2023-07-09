mod firecracker;
mod pyrod_client;

use anyhow::Context;
use anyhow::Result;
pub use firecracker::*;
use notify::{recommended_watcher, Event, EventKind, Watcher};
use pyrod_service::Language;
use std::path::{Path, PathBuf};
use tokio::sync::mpsc;
use tracing::Instrument;

fn get_rootfs(lang: Language) -> PathBuf {
    crate::config::get().resource_path.join(format!(
        "rootfs-{}.ext4",
        match lang {
            Language::Python => "python",
            Language::Rust => "rust",
            Language::Java => todo!(),
        }
    ))
}

#[tracing::instrument(skip(code, input))]
pub async fn run_code(lang: Language, code: String, input: String) -> Result<(String, String)> {
    let config = firecracker::Config {
        cpu_count: 1,
        mem: 1024, //in MiB
        rootfs: get_rootfs(lang),
        kernel: crate::config::get().resource_path.join("kernel.bin"),
    };

    tracing::debug!("Booting new VM...");

    let machine = firecracker::Machine::spawn(config).await?;
    wait_for_file(machine.sock_path())
        .instrument(
            tracing::info_span!("Waiting on VM to boot", pyrod_socket = ?machine.sock_path()),
        )
        .await?;

    tracing::debug!("VM booted, socket at {:?}", machine.sock_path());

    tracing::debug!(code, stdin = input);
    let output = pyrod_client::run_code(machine.sock_path(), lang, code, input).await?;
    tracing::debug!(stdout = output.0, stdin = output.1);

    Ok(output)
}

#[tracing::instrument(ret)]
async fn wait_for_file(file: impl AsRef<Path> + std::fmt::Debug) -> Result<()> {
    let file = file.as_ref();
    let (tx, mut rx) = mpsc::channel(1);

    recommended_watcher(move |res| {
        //will only panic if `rx` half is closed.
        tx.blocking_send(res)
            .expect("Failed to send file notify event")
    })?
    .watch(
        file.parent() //will not panic unless path is bad enough that whole thing is fucked
            .expect("Could not get parent of socket path"),
        notify::RecursiveMode::NonRecursive,
    )?;

    while let Some(res) = rx.recv().await {
        match res {
            Ok(Event {
                kind: EventKind::Create(_),
                paths,
                attrs: _,
            }) if paths == [file] => return Ok(()),
            Ok(e) => tracing::trace!("got file event while watching {:?}, {:?}", file, e),
            Err(e) => return Err(e).context("Notify error while watching file"),
        }
    }
    Ok(())
}
