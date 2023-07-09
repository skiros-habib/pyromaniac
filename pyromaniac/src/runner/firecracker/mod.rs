mod config;
pub use config::VmConfig;

use anyhow::{Context, Result};
use std::{
    os::fd::{FromRawFd, IntoRawFd},
    path::PathBuf,
    process::Stdio,
};
use tempfile::TempDir;
use tokio::process::{Child, Command};

/// Holds the resources for our virtual machine
/// When dropped, will kill the process and clean up all temp resources
pub struct Machine {
    _process: Child,
    dir: TempDir,
}

impl Machine {
    #[tracing::instrument]
    pub async fn spawn(conf: VmConfig) -> Result<Self> {
        //create directory to put all our shit in
        let tempdir = TempDir::new().context("Failed to create tempdir")?;

        tracing::debug!("Tempdir for new VM created at {:?}", tempdir.path());

        //create config file for this VM
        conf.write_to_file(tempdir.path())
            .await
            .context("Failed to write config file")?;

        tracing::debug!("Config file for VM at {:?} written", tempdir.path());

        //we have to create the logfile before firecracker can use it
        tokio::fs::File::create(tempdir.path().join("firecracker.log"))
            .await
            .context("Unable to create log file")?;

        let rootfs_file_name = conf
            .rootfs
            .file_name()
            .unwrap_or_else(|| panic!("The filename for your rootfs is fucked: {:?}", conf.rootfs));

        let tmp_rootfs_path = tempdir.path().join(rootfs_file_name);

        //create a copy of rootfs
        tokio::fs::copy(conf.rootfs, tmp_rootfs_path)
            .await
            .context("Failed to copy rootfs into tempdir")?;

        tracing::debug!("Rootfs for VM {:?} copied over", tempdir.path());

        //spawn firecracker process
        let child = Command::new(crate::config::get().resource_path.join("firecracker"))
            .current_dir(tempdir.path())
            .arg("--no-api")
            .arg("--config-file")
            .arg("config.json")
            .kill_on_drop(true) //IMPORTANT - for process to be killed
            .stdin(Stdio::null())
            .stdout(if cfg!(debug_assertions) {
                // SAFETY - file is open and valid because we literally just opened it
                unsafe { Stdio::from_raw_fd(std::fs::File::create("vm.out")?.into_raw_fd()) }
            } else {
                Stdio::null()
            })
            .stderr(Stdio::null())
            .spawn()
            .context("Failed to spawn Firecracker process")?;

        tracing::info!("VM with tempdir at path {:?} started", tempdir.path());

        Ok(Machine {
            _process: child,
            dir: tempdir,
        })
    }

    pub fn sock_path(&self) -> PathBuf {
        self.dir.path().join("pyrod.sock_5000")
    }
}
