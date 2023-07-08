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

pub async fn run_code(code: String, input: String) -> Result<(String, String)> {
    let config = firecracker::Config {
        cpu_count: 1,
        mem: 2048, //in MiB
        rootfs: "/home/joey/pyro/resources/rootfs.ext4".into(),
        kernel: "/home/joey/pyro/resources/kernel.bin".into(),
    };

    let machine = firecracker::Machine::spawn(config).await?;
    //TODO: use notify crate to wait for unix sock to be created
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    pyrod_client::ping(machine.sock_path()).await?;
    tracing::info!("Spawned and pinged machine succesfully!");

    Ok(("".to_owned(), "".to_owned()))
}
