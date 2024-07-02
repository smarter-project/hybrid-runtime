use clap::Args;
use containerd_client;
use containerd_client::services::v1::{
    container::Runtime, containers_client::ContainersClient, content_client::ContentClient,
    images_client::ImagesClient, Container, CreateContainerRequest, GetImageRequest,
    ReadContentRequest,
};
use containerd_client::tonic::Request;
use containerd_client::with_namespace;
use flate2::read::GzDecoder;
use futures::TryStreamExt;
use oci_spec::image::{ImageConfiguration, ImageManifest};
use std::fs::{create_dir_all, read_to_string};
use tar::Archive;
//use containerd_client::services::v1::snapshots::{ StatSnapshotRequest, MountsRequest, ViewSnapshotRequest, ListSnapshotsRequest, snapshots_client::SnapshotsClient};
use crate::utils::{
    change_status, check_mcu_exists, check_status, container_exists, FirmwareStatus,
};
use crate::{
    BOARD, FIRMWARE_FILE, FIRMWARE_LIB, HYBRID_DIR, MCU, OVERLAYFS, REMOTEPROC, REMOTEPROC_PATH,
    SNAPSHOT_DB, STATE_FILE,
};
use prost_types::Any;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize)]
pub enum ContainerStatus {
    Created,
    Running,
    Stopped,
}

#[derive(Debug, Args)]
pub struct Create {
    /// Firmware container image name.
    #[clap(required = true)]
    pub image: String,
    /// Container ID
    /// Name of the instance of the container that you are starting.
    /// The name you provide for the container instance must be unique on your host.
    #[clap(required = true)]
    pub container_id: String,
}

impl Create {
    #[tokio::main(flavor = "current_thread")]
    pub async fn create(&self, namespace: &str) -> Result<(), String> {
        // check if a container with same id exists
        let channel = containerd_client::connect("/run/containerd/containerd.sock")
            .await
            .expect("connect failed.");
        match container_exists(self.container_id.as_str(), namespace, channel.clone()).await {
            Ok(_) => return Err("Container with same ID already exists.".to_string()),
            Err(_) => {}
        }
        //create_dir_all(format!("{HYBRID_DIR}")).expect("Unable to create hybrid runtime folder.");
        let mut container_labels: HashMap<String, String> = HashMap::new();
        let name = self.image.clone();
        let req = GetImageRequest { name };
        let req: Request<GetImageRequest> = with_namespace!(req, namespace);
        let resp_image = ImagesClient::new(channel.clone())
            .get(req)
            .await
            .expect("failed to get image")
            .into_inner()
            .image;

        // checks if the image exists
        match resp_image {
            Some(image) => {
                // get image hash
                let mut firmware_name = String::new();
                let mut mcu_path = String::new();
                let digest = image.target.unwrap().digest;
                let req = ReadContentRequest {
                    digest,
                    ..Default::default()
                };
                let req: Request<ReadContentRequest> = with_namespace!(req, namespace);
                let manifest = ContentClient::new(channel.clone())
                    .read(req)
                    .await
                    .expect("Failed to get content from digest")
                    .into_inner()
                    .map_ok(|msg| msg.data)
                    .try_concat()
                    .await
                    .expect("Failed to get manifest from digest");

                let manifest = manifest.as_slice();
                match ImageManifest::from_reader(manifest) {
                    Ok(manifest) => {
                        // Image config: contains rootfs hash and labels
                        let req = ReadContentRequest {
                            digest: manifest.config().digest().to_string(),
                            ..Default::default()
                        };
                        let req: Request<ReadContentRequest> = with_namespace!(req, namespace);
                        let content = ContentClient::new(channel.clone())
                            .read(req)
                            .await
                            .expect("couldnt read blob content")
                            .into_inner()
                            .map_ok(|msg| msg.data)
                            .try_concat()
                            .await
                            .expect("couldnt read blob content");
                        let content = content.as_slice();
                        match ImageConfiguration::from_reader(content) {
                            Ok(content) => {
                                match content.config() {
                                    Some(config) => {
                                        match config.entrypoint() {
                                            Some(entrypoint) => {
                                                firmware_name = entrypoint[0].clone();
                                                container_labels.insert(
                                                    "Firmware name".to_string(),
                                                    entrypoint[0].clone(),
                                                );
                                            }
                                            _ => return Err("No entrypoint specified".to_string()),
                                        };
                                        match config.labels() {
                                                    Some(labels) =>  {
                                                        match labels.get("board") { 
                                                            Some(board) => {
                                                                let f = read_to_string(format!("{BOARD}")).unwrap();
                                                                let board_file = f.trim_matches(char::from(0));
                                                                for line in board_file.lines() {
                                                                    if line == board {
                                                                        container_labels.insert("Board".to_string(), board.to_string());
                                                                    } else {
                                                                        return Err("Image board name label does not match the current board".to_string());
                                                                    }
                                                                }
                                                            }
                                                            None => return Err("Board label not set in container image.".to_string()),
                                                        };
                                                        match labels.get("mcu") { 
                                                            Some(mcu_label) => {
                                                                // matches the correct mcu path with the label, if doesn't exist just return 
                                                                mcu_path  = match check_mcu_exists(REMOTEPROC, mcu_label) {
                                                                    Ok(mcu_path) => mcu_path,
                                                                    Err(e) => return Err(e),
                                                                };
                                                                container_labels.insert("MCU path".to_string(), mcu_path.to_string());
                                                                container_labels.insert("MCU name".to_string(), mcu_label.to_string());
                                                            }
                                                            _ => return Err("MCU label not set in container image.".to_string()),
                                                        };
                                                    }
                                                    _ => return Err("No labels specified. Can't match board name and MCU.".to_string()),
                                        };
                                    }
                                    None => return Err("Couldnt find config".to_string()),
                                };
                            }
                            Err(_) => return Err("couldnt read blob content".to_string()),
                        }
                        // get hash of first layer
                        // The array MUST have the base layer at index 0
                        // the first and only layer contains the firmware
                        let layer_digest = manifest.layers()[0].digest();
                        let req = ReadContentRequest {
                            digest: layer_digest.to_string(),
                            ..Default::default()
                        };
                        let req: Request<ReadContentRequest> = with_namespace!(req, namespace);
                        // gzip elf file stored in layer
                        let gzip_bytes = ContentClient::new(channel.clone())
                            .read(req)
                            .await
                            .expect("failed to get content from digest")
                            .into_inner()
                            .map_ok(|msg| msg.data)
                            .try_concat()
                            .await
                            .expect("something");

                        {
                            let tar = GzDecoder::new(&gzip_bytes[..]);
                            let mut archive = Archive::new(tar);
                            // grap firmware from image layer and unpack it in /lib/firmware
                            archive
                                .unpack(format!("{FIRMWARE_LIB}"))
                                .expect("failed to unpack firmware");
                        }

                        let tar = GzDecoder::new(&gzip_bytes[..]);
                        let mut archive = Archive::new(tar);
                        let firmware_path = archive
                            .entries()
                            .unwrap()
                            .next()
                            .unwrap()
                            .expect("something");
                        let temp = firmware_path.path().unwrap();
                        let firmware_name = temp.file_name().unwrap().to_str().unwrap();
                        container_labels
                            .insert("Firmware path".to_string(), firmware_name.to_string());
                    }
                    Err(_) => return Err("failed to read image manifest".to_string()),
                }

                // TODO: should be done in shim but just to test cli works we'll do it here

                // random info here, we're interested only in {id, labels, image, runtime}
                let runtime_hybrid = Runtime {
                    name: "hybrid".to_string(),
                    ..Default::default()
                };
                let spec = include_str!("container_spec.json");
                let rootfs = "/tmp/busybox/bundle/rootfs";
                // the container will run with command `echo $output`
                let output = "hello rust client";
                let spec = spec
                    .to_string()
                    .replace("$ROOTFS", rootfs)
                    .replace("$OUTPUT", output);

                // spec is runtim specific
                // TODO: figure out the right format for the container spec
                let spec = Any {
                    type_url: "types.containerd.io/opencontainers/runtime-spec/1/Spec".to_string(),
                    value: spec.into_bytes(),
                };

                // since we're using snapshotter for firmware image, we need to specify snapshotter
                // TODO: update snapshotter to point to the right one
                let snapshotter = "temp_snapshotter".to_string();
                let snapshot_key = "temp_key".to_string();
                // put firmware name in labels, is location needed? ... not for now
                // TODO: is it necessary to create the container here??
                let container = Container {
                    id: self.container_id.clone(),
                    labels: container_labels,
                    image: image.name.clone(),
                    runtime: Some(runtime_hybrid),
                    spec: Some(spec),
                    snapshotter: snapshotter,
                    snapshot_key: snapshot_key,
                    ..Default::default()
                };

                let req = CreateContainerRequest {
                    container: Some(container),
                };
                let req: Request<CreateContainerRequest> = with_namespace!(req, namespace);
                let resp = ContainersClient::new(channel.clone())
                    .create(req)
                    .await
                    .expect("failed to get content from digest")
                    .into_inner();

                // create dir for the container under /var/lib/hybrid-runtime/{container_id}
                create_dir_all(format!("{HYBRID_DIR}/{}", self.container_id))
                    .expect("Failed to create container folder.");
            }
            None => return Err("no image found".to_string()),
        }
        Ok(())
    }
}
