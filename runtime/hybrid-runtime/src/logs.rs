use crate::utils::container_exists;
use crate::HYBRID_DIR;
use clap::Args;
use containerd_client::connect;
use std::fs::read_to_string;

#[derive(Debug, Args)]
pub struct Logs {
    /// Container ID
    /// Name of the container
    #[clap(required = true)]
    pub container_id: String,
}

impl Logs {
    #[tokio::main(flavor = "current_thread")]
    pub async fn logs(&self, namespace: &str) -> Result<(), String> {
        // check if the container exists
        let channel = connect("/run/containerd/containerd.sock")
            .await
            .expect("Connection to containerd failed.");
        match container_exists(self.container_id.as_str(), namespace, channel.clone()).await {
            Ok(_) => {
                // For the moment use rpmsg console app to write logs
                // This assumes the logs were already written by the rpmsg console app
                let logs = read_to_string(format!(
                    "{HYBRID_DIR}/{0}/{0}.log",
                    self.container_id.clone()
                ))
                .expect("Couldn't open log file.");
                println!("{logs}");
            }
            Err(_) => return Err("No container with that ID.".to_string()),
        }

        Ok(())
    }
}
