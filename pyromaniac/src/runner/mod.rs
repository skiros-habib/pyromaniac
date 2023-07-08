mod firecracker;
mod pyrod_client;

use anyhow::Result;
pub use firecracker::*;
use pyrod_service::Language;
use std::path::PathBuf;

fn get_roofs(lang: Language) -> PathBuf {
    match lang {
        Language::Python => todo!(),
        Language::Rust => todo!(),
        Language::Java => todo!(),
    }
}

#[tracing::instrument(skip(code, input))]
pub async fn run_code(lang: Language, code: String, input: String) -> Result<(String, String)> {
    let config = firecracker::Config {
        cpu_count: 1,
        mem: 2048, //in MiB
        rootfs: "/home/joey/pyro/resources/rootfs-python.ext4".into(),
        kernel: "/home/joey/pyro/resources/kernel.bin".into(),
    };

    tracing::debug!("Booting new VM...");

    let machine = firecracker::Machine::spawn(config).await?;

    //TODO: use notify crate to wait for unix sock to be created
    {
        let _span =
            tracing::info_span!("Waiting on VM to boot", pyrod_socket = ?machine.sock_path())
                .entered();
        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
    }
    tracing::debug!("VM booted, socket at {:?}", machine.sock_path());

    tracing::debug!(code, stdin = input);
    let output = pyrod_client::run_code(machine.sock_path(), lang, code, input).await?;
    tracing::debug!(stdout = output.0, stdin = output.1);

    Ok(output)
}
