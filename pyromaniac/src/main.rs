use std::time::Duration;

mod executor;
mod vm_client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing::subscriber::set_global_default(
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .finish(),
    )?;
    let kernel = "resources/kernel.bin";
    let rootfs = "resources/rootfs.ext4";
    let firecracker =
        "/home/joey/firecracker/build/cargo_target/x86_64-unknown-linux-musl/debug/firecracker";
    executor::Executor::spawn(kernel.as_ref(), rootfs.as_ref(), firecracker.as_ref(), 1).await?;
    tokio::time::sleep(Duration::from_secs(100)).await;

    Ok(())
}
