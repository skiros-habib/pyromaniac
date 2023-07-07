use std::path::{Path, PathBuf};

use anyhow::Result;
use firepilot::{
    builder::{
        drive::DriveBuilder, executor::FirecrackerExecutorBuilder, kernel::KernelBuilder,
        network_interface::NetworkInterfaceBuilder, Builder, Configuration,
    },
    machine::Machine,
};

#[derive(Debug)]
enum ExecutorState {
    Booting,
    Idling,
    Executing,
}

#[derive(Debug)]
pub struct Executor {
    exec_dir: tempfile::TempDir,
    machine: Machine,
    state: ExecutorState,
    id: u64,
}

impl Executor {
    pub async fn spawn(kernel: &Path, rootfs: &Path, firecracker: &Path, id: u64) -> Result<Self> {
        let kernel = KernelBuilder::new()
            .with_kernel_image_path(kernel.to_string_lossy().to_string())
            .with_boot_args("reboot=k panic=1 pci=off random.trust_cpu=on".to_string())
            .try_build()?;

        let drive = DriveBuilder::new()
            .with_drive_id("rootfs".to_string())
            .with_path_on_host(rootfs.to_owned())
            .as_read_only()
            .as_root_device()
            .try_build()?;

        let exec_dir = tempfile::tempdir()?;

        let executor = FirecrackerExecutorBuilder::new()
            .with_chroot(exec_dir.path().to_string_lossy().to_string())
            .with_exec_binary(PathBuf::from(firecracker))
            .try_build()
            .unwrap();

        let iface = NetworkInterfaceBuilder::new()
            .with_iface_id("eth0".to_string())
            .with_host_dev_name(format!("tap{}", id))
            .try_build()
            .unwrap();

        let config = Configuration::new(format!("pyro_executor_{}", id))
            .with_kernel(kernel)
            .with_executor(executor)
            .with_drive(drive)
            .with_interface(iface);

        tracing::debug!("Created VM id: {} config: {:?}", id, config);

        let mut machine = Machine::new();
        machine.create(config).await?;
        machine.start().await?;

        tracing::debug!(
            "Firecracker VM with id {} successfully span up: {:?}",
            id,
            machine
        );

        Ok(Executor {
            exec_dir,
            id,
            machine,
            state: ExecutorState::Idling,
        })
    }

    // pub async fn ping() -> Result<ExecutorState> {}

    // pub async fn execute() -> Result<()> {}
}
