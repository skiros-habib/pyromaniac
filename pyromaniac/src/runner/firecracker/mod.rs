mod config;
pub use config::VmConfig;

use anyhow::{Context, Result};
use std::os::unix::fs::PermissionsExt;
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
    _dir: TempDir,
    pub chroot: PathBuf,
}

impl Machine {
    //the spawn methods are very different for debug and release due to how jailer
    //uses cgroups to restrict filesystem access
    //see https://github.com/firecracker-microvm/firecracker/issues/1477
    //when using jailer, we have to put everything at
    // /tmp/<tempdir>/firecracker/<vm_id>/root
    // can always use the same vm_id because we're using different tempdir roots
    pub async fn spawn(conf: VmConfig) -> Result<Self> {
        //create directory to put all our shit in
        let tempdir = TempDir::new().context("Failed to create tempdir")?;

        let chroot = if cfg!(debug_assertions) {
            tempdir.path().into()
        } else {
            tempdir.path().join("firecracker").join("1").join("root")
        };

        std::fs::create_dir_all(&chroot)
            .context(format!("Failed to create chroot dir at {:?}", chroot))?;

        tracing::debug!("Tempdir for new VM created at {:?}", chroot);

        //create config file for this VM
        conf.write_to_file(&chroot)
            .await
            .context("Failed to write config file")?;

        tracing::debug!("Config file for VM at {:?} written", &chroot);

        //we have to create the logfile before firecracker can use it
        tokio::fs::File::create(&chroot.join("firecracker.log"))
            .await
            .context("Unable to create log file")?;

        let rootfs_file_name = conf
            .rootfs
            .file_name()
            .unwrap_or_else(|| panic!("The filename for your rootfs is fucked: {:?}", conf.rootfs));

        let tmp_rootfs_path = chroot.join(rootfs_file_name);

        //create a copy of rootfs
        tokio::fs::copy(&conf.rootfs, tmp_rootfs_path)
            .await
            .context("Failed to copy rootfs into tempdir")?;

        tracing::debug!("Rootfs for VM at {:?} copied over", chroot);

        //we need to copy kernel into chroot so firecracker can use it when running in jailer mode
        //to save a copy we can just hard link it
        //we *do* have to copy rootfs tho because those are modified between runs
        std::fs::hard_link(
            crate::config::get().resource_path.join("kernel.bin"),
            chroot.join("kernel.bin"),
        )
        .context("Failed to hard link kernel into chroot")?;

        //mark kernel as executable by anyone (firecracker runs under different uid)
        //TODO: chown instead?
        std::fs::set_permissions(
            chroot.join("kernel.bin"),
            std::fs::Permissions::from_mode(0o777),
        )
        .expect("Could not set perms for kernel");

        //mark fs as writable
        std::fs::set_permissions(
            chroot.join(rootfs_file_name),
            std::fs::Permissions::from_mode(0o777),
        )
        .expect("Could not set perms for rootfs");

        //spawn firecracker process
        //use jailer in release mode, firecracker in debug
        let child = if cfg!(debug_assertions) {
            Command::new(crate::config::get().resource_path.join("firecracker"))
                .current_dir(&chroot)
                .arg("--no-api")
                .arg("--config-file")
                .arg(&chroot.join("config.json"))
                .kill_on_drop(true) //IMPORTANT - for process to be killed
                .stdin(Stdio::null())
                .stdout(unsafe {
                    //SAFETY - file is open and valid because we just opened it
                    Stdio::from_raw_fd(std::fs::File::create("vm.out")?.into_raw_fd())
                })
                .stderr(unsafe {
                    Stdio::from_raw_fd(std::fs::File::create("vm.err")?.into_raw_fd())
                })
                .spawn()
                .context("Failed to spawn Firecracker process")?
        } else {
            Command::new(crate::config::get().resource_path.join("jailer"))
                .current_dir(&chroot)
                .arg("--id")
                .arg("1")
                .arg("--exec-file")
                .arg(crate::config::get().resource_path.join("firecracker"))
                .arg("--uid")
                .arg(
                    crate::config::get()
                        .runner_config
                        .uid
                        .expect("No uid provided, cannot start jailer")
                        .to_string(),
                )
                .arg("--gid")
                .arg(
                    crate::config::get()
                        .runner_config
                        .uid
                        .expect("No gid provided, cannot start jailer")
                        .to_string(),
                )
                .arg("--chroot-base-dir")
                .arg(tempdir.path()) //actual chroot is base_dir/firecracker/vm/root
                .arg("--") //firecracker args go after this
                .arg("--no-api")
                .arg("--config-file")
                .arg("config.json")
                .kill_on_drop(true) //IMPORTANT - for process to be killed
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .context("Failed to spawn Jailer/Firecracker process")?
        };

        tracing::info!("VM at path {:?} started", chroot);

        //check firecracker did not insta exit
        let machine = Machine {
            _process: child,
            _dir: tempdir,
            chroot,
        };

        Ok(machine)
    }
}
