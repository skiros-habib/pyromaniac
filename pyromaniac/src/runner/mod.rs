mod firecracker;
mod pyrod_client;

use anyhow::Result;
pub use firecracker::*;
use pyrod_service::Language;
use std::path::PathBuf;

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
    let config = firecracker::VmConfig {
        runner: &crate::config::get().runner_config,
        rootfs: get_rootfs(lang),
        kernel: crate::config::get().resource_path.join("kernel.bin"),
    };

    tracing::debug!("Booting new VM...");

    let machine = firecracker::Machine::spawn(config).await?;

    tracing::debug!("VM process spawned, socket at {:?}", machine.sock_path());

    let output = pyrod_client::run_code(machine.sock_path(), lang, code, input).await?;

    Ok(output)
}
