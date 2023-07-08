use anyhow::Result;
use anyhow::{anyhow, Context};
use serde_json::json;
use std::path::{Path, PathBuf};
use tokio::io::AsyncWriteExt;

pub struct Config {
    pub cpu_count: u32,
    pub mem: u32,
    pub rootfs: PathBuf,
    pub kernel: PathBuf,
}

impl Config {
    pub async fn write_to_file<P: AsRef<Path>>(&self, tmp_path: P) -> Result<()> {
        let tmp_path = tmp_path.as_ref();

        //generate all the json that we need to dump to file

        let boot_source = json!({
            "kernel_image_path": self.kernel,
            "boot_args": "reboot=k panic=1 pci=off random.trust_cpu=on"
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
            "vcpu_count": self.cpu_count,
            "mem_size_mib": self.mem,
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
