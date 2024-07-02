use containerd_client;
use containerd_client::services::v1::{
    container::Runtime, containers_client::ContainersClient, content_client::ContentClient,
    images_client::ImagesClient, Container, CreateContainerRequest, GetContainerRequest,
    GetImageRequest, ReadContentRequest, UpdateContainerRequest,
};
use containerd_client::tonic::Request;
use containerd_client::with_namespace;
use futures::TryStreamExt;
use oci_spec::image::{ImageConfiguration, ImageManifest};
use std::fs::{create_dir_all, read_to_string};
//use containerd_client::services::v1::snapshots::{ StatSnapshotRequest, MountsRequest, ViewSnapshotRequest, ListSnapshotsRequest, snapshots_client::SnapshotsClient};
use crate::utils::check_mcu_exists;
use crate::{BOARD, HYBRID_DIR, NAMESPACE, REMOTEPROC, REMOTEPROC_PATH};
use prost_types::Any;
use std::collections::HashMap;
use std::fs::write;
use log::info;


#[tokio::main(flavor = "current_thread")]
pub async fn create_container(
    container_id: &str,
    firmware_path: &str,
) -> Result<(), String> {

//) -> Result<Container, String> {

    info!("connect to containerd \n\n");
    let channel = containerd_client::connect("/run/containerd/containerd.sock")
        .await
        .expect("connect failed.");
    let mut container_labels: HashMap<String, String> = HashMap::new();
    let req = GetContainerRequest {
        id: container_id.to_string(),
    };

    info!("get container to retreive image");
    let req: Request<GetContainerRequest> = with_namespace!(req, NAMESPACE);
    let current_container = ContainersClient::new(channel.clone())
        .get(req)
        .await
        .expect("failed to get image")
        .into_inner()
        .container
        .unwrap();


    let name = current_container.clone().image;

    let req = GetImageRequest {
        name: name.to_string(),
    };

    info!("get image");
    let req: Request<GetImageRequest> = with_namespace!(req, NAMESPACE);
    let image = ImagesClient::new(channel.clone())
        .get(req)
        .await
        .expect("failed to get image")
        .into_inner()
        .image
        .unwrap();

    let mut mcu_path = String::new();
    let digest = image.target.unwrap().digest;
    let req = ReadContentRequest {
        digest,
        ..Default::default()
    };
    let req: Request<ReadContentRequest> = with_namespace!(req, NAMESPACE);
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

    info!("get image manifest");
    match ImageManifest::from_reader(manifest) {
        Ok(manifest) => {
            // Image config: contains rootfs hash and labels
            // for shim I only need to extract labels
            let req = ReadContentRequest {
                digest: manifest.config().digest().to_string(),
                ..Default::default()
            };
            let req: Request<ReadContentRequest> = with_namespace!(req, NAMESPACE);
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
            info!("image configuration");
            match ImageConfiguration::from_reader(content) {
                Ok(content) => {
                    match content.config() {
                        Some(config) => {
                            info!("getting entrypoint");
                            match config.entrypoint() {
                                Some(entrypoint) => {
                                    // I could get the firmaware name from firmware path
                                    // entrypoint not necessary
                                    container_labels
                                        .insert("Firmware name".to_string(), entrypoint[0].clone());
                                }
                                _ => return Err("No entrypoint specified".to_string()),
                            };
                            info!("getting labels");
                            match config.labels() {
                                Some(labels) => {
                                    match labels.get("board") {
                                        Some(board) => {
                                            let f = read_to_string(format!("{BOARD}")).unwrap();
                                            let board_file = f.trim_matches(char::from(0));
                                            for line in board_file.lines() {
                                                if line == board {
                                                    container_labels.insert(
                                                        "Board".to_string(),
                                                        board.to_string(),
                                                    );
                                                } else {
                                                    return Err("Image board name label does not match the current board".to_string());
                                                }
                                            }
                                        }
                                        None => {
                                            return Err("Board label not set in container image."
                                                .to_string())
                                        }
                                    };
                                    match labels.get("mcu") {
                                        Some(mcu_label) => {
                                            mcu_path = match check_mcu_exists(REMOTEPROC, mcu_label)
                                            {
                                                Ok(mcu_path) => mcu_path,
                                                Err(e) => return Err(e),
                                            };
                                            container_labels.insert(
                                                "MCU path".to_string(),
                                                mcu_path.to_string(),
                                            );
                                            container_labels.insert(
                                                "MCU name".to_string(),
                                                mcu_label.to_string(),
                                            );
                                        }
                                        _ => {
                                            return Err(
                                                "MCU label not set in container image.".to_string()
                                            )
                                        }
                                    };
                                }
                                _ => {
                                    return Err(
                                        "No labels specified. Can't match board name and MCU."
                                            .to_string(),
                                    )
                                }
                            };
                        }
                        None => return Err("Couldnt find config".to_string()),
                    };
                }
                Err(_) => return Err("couldnt read blob content".to_string()),
            }
            // need to update where remoteproc looks for firmware
            write(REMOTEPROC_PATH, firmware_path)
                .expect("Couldn't update firmware default location.");
            container_labels.insert("Firmware path".to_string(), firmware_path.to_string());
        }
        Err(_) => return Err("failed to read image manifest".to_string()),
    }

    // random info here, we're interested only in {id, labels, image, runtime}
   /* let runtime_hybrid = Runtime {
        name: "io.containerd.hybrid".to_string(),
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

    // spec is runtime specific
    // TODO: figure out the right format for the container spec
    let spec = Any {
        type_url: "types.containerd.io/opencontainers/runtime-spec/1/Spec".to_string(),
        value: spec.into_bytes(),
    };

    // since we're using snapshotter for firmware image, we need to specify snapshotter
    // TODO: update snapshotter to point to the right one
    let snapshotter = "temp_snapshotter".to_string();
    let snapshot_key = "temp_key".to_string();
    
    let container = Container {
        id: container_id.to_string(),
        labels: container_labels,
        image: image.name.clone(),
        runtime: Some(runtime_hybrid),
        spec: Some(spec),
        snapshotter: snapshotter,
        snapshot_key: snapshot_key,
        ..Default::default()
    };

*/

    // update current container
    let container = Container {
        labels: container_labels,
        ..current_container
    };

    let req = UpdateContainerRequest {
        container: Some(container.clone()),
        //the fields to perform the update on: labels
        update_mask: None,
    };
    let req: Request<UpdateContainerRequest> = with_namespace!(req, NAMESPACE);
    let resp = ContainersClient::new(channel.clone())
        .update(req)
        .await
        .expect("failed to update current container")
        .into_inner();

    // no need to create a container here (ctr takes care of this)
/* *
    let req = CreateContainerRequest {
        container: Some(current.clone()),
    };
    let req: Request<CreateContainerRequest> = with_namespace!(req, NAMESPACE);
    let resp = ContainersClient::new(channel.clone())
        .create(req)
        .await
        .expect("failed to get content from digest")
        .into_inner();
*/
    // create dir for the container under /var/lib/hybrid-runtime/{container_id}
    create_dir_all(format!("{HYBRID_DIR}/{}", container_id))
        .expect("Failed to create container folder.");

    Ok(())
}
