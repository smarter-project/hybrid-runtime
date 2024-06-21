use crate::utils::{change_status, check_status, container_exists, FirmwareStatus};
use crate::{HYBRID_DIR, STATE_FILE};
use clap::Args;
use containerd_client::{
    connect,
    services::v1::{containers_client::ContainersClient, DeleteContainerRequest},
    tonic::Request,
    with_namespace,
};
use std::fs::remove_dir_all;

#[derive(Debug, Args)]
pub struct Delete {
    /// container ID
    #[clap(required = true)]
    pub container_id: String,
    /// Forcibly deletes the container if it is still running (using SIGKILL)
    #[arg(short, long)]
    pub force: bool,
}

impl Delete {
    #[tokio::main(flavor = "current_thread")]
    pub async fn delete(&self, namespace: &str) -> Result<(), String> {
        let channel = connect("/run/containerd/containerd.sock")
            .await
            .expect("connect failed.");
        match container_exists(self.container_id.as_str(), namespace, channel.clone()).await {
            Ok(container) => {
                let labels = container.into_inner().container.unwrap().labels;
                let mcu_path = labels.get("MCU path").unwrap();
                let req = DeleteContainerRequest {
                    id: self.container_id.clone(),
                };
                let req: Request<DeleteContainerRequest> = with_namespace!(req, namespace);
                let _resp = ContainersClient::new(channel.clone())
                    .delete(req)
                    .await
                    .expect("Failed to delete container.");
                println!("container with id: {:?} deleted", self.container_id);
                match check_status(format!("{mcu_path}/{STATE_FILE}").as_str()).unwrap() {
                    FirmwareStatus::Offline => (),
                    FirmwareStatus::Running => change_status(
                        format!("{mcu_path}/{STATE_FILE}").as_str(),
                        FirmwareStatus::Offline,
                    ),
                }
                remove_dir_all(format!("{HYBRID_DIR}/{}", self.container_id.clone()))
                    .expect("Failed to delete container resources (logs).");
            }
            Err(_) => return Err("No container with ID {self.container_id}".to_string()),
        }
        Ok(())
    }
}
