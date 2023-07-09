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
    pub async fn write_to_file<P: AsRef<Path> + std::fmt::Debug>(&self, tmp_path: P) -> Result<()> {
        let tmp_path = tmp_path.as_ref();

        //generate all the json that we need to dump to file

        //disable stty if we don't need it, only used for writing logs in debug mode
        let boot_args = if cfg!(debug_assertions) {
            "init=/bin/pyrod console=ttyS0 reboot=k panic=1 pci=off"
        } else {
            "init=/bin/pyrod reboot=k panic=1 pci=off"
        };

        let boot_source = json!({
            "kernel_image_path": self.kernel,
            "boot_args":boot_args,
        });

        let drive = {
            let file_name = self.rootfs.file_name().unwrap_or_else(|| {
                panic!("The filename for your rootfs is fucked: {:?}", self.rootfs)
            });
            let tmp_rootfs_path = tmp_path.join(file_name);

            json!({
                "drive_id": "rootfs",
                "path_on_host": tmp_rootfs_path,
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
            "uds_path": tmp_path.join("pyrod.sock"),
            "vsock_id": "vsock0"
        });
        let logger = json!({
            "log_path": tmp_path.join("firecracker.log"),
            "level": "Debug",
            "show_level": true,
            "show_log_origin": true
        });

        //actual final json object
        let config_json = json!({
            "boot-source": boot_source,
            "drives": [drive],
            "machine-config": machine_config,
            "vsock": vsock,
            "logger": logger
        });

        //dump it to file
        let config_path = tmp_path.join("config.json");
        let mut outfile = tokio::fs::File::create(&config_path)
            .await
            .context(format!("Could not open config file {config_path:?}"))?;

        outfile
            .write_all(config_json.to_string().as_bytes())
            .await
            .context(format!("Could not open config file {config_path:?}"))
    }
}
