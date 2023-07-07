use std::time::Duration;

mod executor;
mod vm_client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    let kernel = "resources/kernel.bin";
    let rootfs = "resources/rootfs.ext4";
    let firecracker =
        "/home/joey/firecracker/build/cargo_target/x86_64-unknown-linux-musl/debug/firecracker";
    // executor::Executor::spawn(kernel.as_ref(), rootfs.as_ref(), firecracker.as_ref(), 1).await?;
    // tokio::time::sleep(Duration::from_secs(100)).await;
    vm_client::send_request("/home/joey/pyro/v.sock").await?;
    Ok(())
}
