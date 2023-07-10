use anyhow::Context;
use anyhow::Result;
use serde_json::json;
use std::path::{Path, PathBuf};
use tokio::io::AsyncWriteExt;

#[derive(Debug)]
pub struct VmConfig {
    pub rootfs: PathBuf,
    pub kernel: PathBuf,
    pub runner: &'static crate::config::RunnerConfig,
}

impl VmConfig {
    #[tracing::instrument]
    pub async fn write_to_file<P: AsRef<Path> + std::fmt::Debug>(&self, chroot: P) -> Result<()> {
        let chroot = chroot.as_ref();

        //generate all the json that we need to dump to file

        //disable stty if we don't need it, only used for writing logs in debug mode
        let boot_args = if cfg!(debug_assertions) {
            "init=/bin/pyrod console=ttyS0 reboot=k panic=1 pci=off"
        } else {
            "init=/bin/pyrod reboot=k panic=1 pci=off"
        };

        let boot_source = json!({
            "kernel_image_path": "kernel.bin",
            "boot_args":boot_args,
        });

        let drive = {
            let file_name = self
                .rootfs
                .file_name()
                .and_then(|f| f.to_str())
                .unwrap_or_else(|| {
                    panic!("The filename for your rootfs is fucked: {:?}", self.rootfs)
                });

            json!({
                "drive_id": "rootfs",
                "path_on_host": &file_name,
                "is_root_device": true,
                "is_read_only": false
            })
        };

        let machine_config = json!({
            "vcpu_count": self.runner.cpus,
            "mem_size_mib": self.runner.memory,
            "smt": false
        });
        let vsock = json!({
            "guest_cid": 3,
            "uds_path": "pyrod.sock",
            "vsock_id": "vsock0",
        });

        let logger = json!({
            "log_path": "firecracker.log",
            "level": "Debug",
            "show_level": true,
            "show_log_origin": true
        });

        //don't write logs in release mode
        let config_json = if cfg!(debug_assertions) {
            //actual final json object
            json!({
                "boot-source": boot_source,
                "drives": [drive],
                "machine-config": machine_config,
                "vsock": vsock,
                "logger": logger
            })
        } else {
            json!({
                "boot-source": boot_source,
                "drives": [drive],
                "machine-config": machine_config,
                "vsock": vsock
            })
        };

        //dump it to file
        let config_path = chroot.join("config.json");
        let mut outfile = tokio::fs::File::create(&config_path)
            .await
            .context(format!("Could not open config file {config_path:?}"))?;

        outfile
            .write_all(config_json.to_string().as_bytes())
            .await
            .context(format!("Could not open config file {config_path:?}"))
    }
}
