use crate::utils::{change_status, check_status, container_exists, FirmwareStatus};
use crate::{CONSOLE_RPMSG, FIRMWARE_FILE, HYBRID_DIR, STATE_FILE};
use clap::Args;
use std::fs::{write, File};
use std::process::Command;

#[derive(Debug, Args)]
pub struct Start {
    /// container ID
    #[clap(required = true)]
    pub container_id: String,
}

impl Start {
    #[tokio::main(flavor = "current_thread")]
    pub async fn start(&self, namespace: &str) -> Result<(), String> {
        let channel = containerd_client::connect("/run/containerd/containerd.sock")
            .await
            .expect("connect failed.");
        match container_exists(self.container_id.as_str(), namespace, channel).await {
            Ok(container) => {
                let labels = container.into_inner().container.unwrap().labels;
                let mcu_path = labels.get("MCU path").unwrap();
                let firmware_name = labels.get("Firmware name").unwrap();
                let _ = File::create(format!("{HYBRID_DIR}/{}", self.container_id.clone()));

                match check_status(format!("{mcu_path}/{STATE_FILE}").as_str()).unwrap() {
                    FirmwareStatus::Offline => {
                        println!("state offline => starting firmware...");
                        write(format!("{mcu_path}/{FIRMWARE_FILE}"), firmware_name)
                            .expect("Unable to write firmware name to file.");
                        change_status(
                            format!("{mcu_path}/{STATE_FILE}").as_str(),
                            FirmwareStatus::Running,
                        );
                    }
                    FirmwareStatus::Running => {
                        return Err(
                            "Can'r start container, a firmware is already running.".to_string()
                        )
                    }
                }

                Command::new(format!("{CONSOLE_RPMSG}"))
                    .arg(format!(
                        "{HYBRID_DIR}/{0}/{0}.log",
                        self.container_id.clone()
                    ))
                    .spawn()
                    .expect("Failed to run rpmsg console");
            }
            Err(_) => return Err("No container with ID {self.container_id}".to_string()),
        }
        Ok(())
    }
}
