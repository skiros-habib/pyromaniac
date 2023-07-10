mod firecracker;
mod pyrod_client;

use anyhow::Result;
pub use firecracker::*;
use pyrod_service::Language;
use std::path::PathBuf;

fn get_rootfs(lang: Language) -> PathBuf {
    crate::config::get()
        .resource_path
        .join(format!("rootfs-{lang}.ext4"))
}

#[tracing::instrument(skip(code, input))]
pub async fn run_code(lang: Language, code: String, input: String) -> Result<(String, String)> {
    let config = firecracker::VmConfig {
        runner: &crate::config::get().runner_config,
        rootfs: get_rootfs(lang),
        kernel: crate::config::get().resource_path.join("kernel.bin"),
    };

    tracing::debug!("Booting new VM...");

    let machine = firecracker::Machine::spawn(config).await?;

    tracing::debug!("VM process spawned, chroot at {:?}", machine.chroot());

    let output =
        pyrod_client::run_code(machine.chroot().join("pyrod.sock_5000"), lang, code, input).await?;

    Ok(output)
}
