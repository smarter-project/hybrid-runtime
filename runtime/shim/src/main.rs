use crate::utils::{change_status, check_mcu_exists, check_status, read_info, FirmwareStatus};
use containerd_client;
use std::sync::Arc;

use containerd_client::services::v1::{
    containers_client::ContainersClient, content_client::ContentClient,
    images_client::ImagesClient, Container, GetContainerRequest, GetImageRequest,
    ReadContentRequest, UpdateContainerRequest,
};
use containerd_client::tonic::Request;
use containerd_client::with_namespace;
use containerd_shim as shim;
use futures::TryStreamExt;
use log::{error, info};
use oci_spec::image::{ImageConfiguration, ImageManifest};
use shim::{
    api,
    api::{
        CreateTaskRequest, CreateTaskResponse, DeleteRequest, DeleteResponse, Empty, KillRequest,
        StartRequest, StartResponse, StateRequest, StateResponse, StatsRequest, StatsResponse,
        Status, WaitRequest, WaitResponse,
    },
    protos::events::task::{TaskCreate, TaskDelete, TaskIO, TaskStart},
    synchronous::publisher::RemotePublisher,
    util::convert_to_timestamp,
    Config, Context, ExitSignal, Flags, TtrpcContext, TtrpcResult,
};
use std::collections::HashMap;
use std::fs::{create_dir_all, read_to_string, remove_dir_all, remove_file, write, File};
use std::path::Path;
use std::process::Command;
use time::OffsetDateTime;
//use std::process::Stdio;
use notify::{Config as notify_config, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::PathBuf;
use std::sync::mpsc;

mod utils;

pub const REMOTEPROC_PATH: &str = "/sys/module/firmware_class/parameters/path";
pub const REMOTEPROC: &str = "/sys/class/remoteproc";
pub const STATE_FILE: &str = "state";
pub const FIRMWARE_FILE: &str = "firmware";
pub const HYBRID_DIR: &str = "/var/lib/hybrid-runtime";
pub const CONSOLE_RPMSG: &str = "/usr/local/bin/cortexm_console";
pub const BOARD: &str = "/sys/firmware/devicetree/base/model";
pub const MCU: &str = "name";

#[derive(Clone)]
pub(crate) struct Service {
    pub exit: Arc<ExitSignal>,
    pub namespace: String,
    pub address: String,
}

impl shim::Shim for Service {
    type T = Service;

    fn new(_runtime_id: &str, args: &Flags, _config: &mut Config) -> Self {
        Service {
            exit: Arc::new(ExitSignal::default()),
            namespace: args.namespace.to_string(),
            address: args.address.to_string(),
        }
    }

    fn start_shim(&mut self, opts: shim::StartOpts) -> Result<String, shim::Error> {
        let grouping = opts.id.clone();
        let (_child_id, address) = shim::spawn(opts, &grouping, Vec::new())?;
        Ok(address)
    }

    fn delete_shim(&mut self) -> Result<DeleteResponse, shim::Error> {
        Ok(DeleteResponse::new())
    }

    fn wait(&mut self) {
        self.exit.wait();
    }

    fn create_task_service(&self, _publisher: RemotePublisher) -> Self::T {
        self.clone()
    }
}

impl shim::Task for Service {
    #[tokio::main]
    async fn create(
        &self,
        _ctx: &TtrpcContext,
        request: CreateTaskRequest,
    ) -> TtrpcResult<CreateTaskResponse> {
        let ns = self.namespace.clone();
        let address = self.address.clone();
        let container_id: &str = request.id.as_str();
        let _bundle_path: &str = request.bundle.as_str();
        let rootfs = &request.rootfs[0].options;
        let firmware_path = &rootfs[rootfs.iter().position(|x| x.contains("lowerdir=")).unwrap()]
            .strip_prefix("lowerdir=")
            .unwrap();

        let channel = containerd_client::connect(address.clone())
            .await
            .expect("connect failed.");
        let mut container_labels: HashMap<String, String> = HashMap::new();
        let req = GetContainerRequest {
            id: container_id.to_string(),
        };

        let req: Request<GetContainerRequest> = with_namespace!(req, ns);
        let current_container = ContainersClient::new(channel.clone())
            .get(req)
            .await
            .expect("failed to get image")
            .into_inner()
            .container
            .unwrap();

        let name = current_container.clone().image;

        let mut pid = 0;
        let pid_file = read_to_string("/proc/sys/kernel/ns_last_pid").unwrap();
        for line in pid_file.lines() {
            pid = line.parse::<u32>().unwrap() + 10;
        }
        create_dir_all(format!("{HYBRID_DIR}/{}", container_id))
            .expect("Failed to create container folder.");

        let mut resp = CreateTaskResponse::new();
        info!("create a container using hybrid-runtime.");
        // k3s Hack: check if it's a pause container
        // name: docker.io/rancher/mirrored-pause:3.6
        if name.contains("pause") {
            info!("found a pause container");
            // delete the pause container that was created using the hybrid-runtime
            Command::new("ctr")
                .arg("c")
                .arg("rm")
                .arg(container_id.to_string())
                .spawn()
                .expect("failed to execute process");
            info!("delete original pause");
            // create a new pause container using runc wiht the same ID
            Command::new("/usr/local/bin/pause.sh")
                .arg(name)
                .arg(container_id.to_string())
                .spawn()
                .expect("failed to execute process");

            info!("container started using runc.");

            let cmd = format!(
                "echo $(ps -e -o pid,cmd | grep {} | awk",
                container_id.to_string()
            ) + " 'NR==1{print $1}') > ";
            let pause_cmd = cmd.clone() + format!("{HYBRID_DIR}/{}/pause", container_id).as_str();
            info!("command being executed {:?}", pause_cmd.clone());
            Command::new("sh")
                .arg("-c")
                .arg(pause_cmd)
                .spawn()
                .expect("failed to execute process");
            let (tx, rx) = mpsc::channel();
            let mut watcher = RecommendedWatcher::new(tx, notify_config::default()).unwrap();
            let file_path = PathBuf::from(format!("{HYBRID_DIR}/{}/pause", container_id));
            let file_dir = file_path.parent().unwrap();
            info!("file dir (should be parent) {:?}", file_dir);
            watcher
                .watch(&file_dir, RecursiveMode::NonRecursive)
                .unwrap();
            if !file_path.exists() {
                for res in rx {
                    match res {
                        Ok(notify::event::Event {
                            kind: notify::event::EventKind::Create(_),
                            paths: p,
                            ..
                        }) => {
                            info!("A file was created: {p:?}");
                            if p.first() == Some(&file_path) {
                                info!("pause file was found");
                                break;
                            }
                        }
                        Ok(_) => continue,
                        Err(error) => error!("Error: {error:?}"),
                    }
                }
            }
            watcher.unwatch(file_dir).unwrap();

            let pid = read_info(format!("{HYBRID_DIR}/{}/pause", request.id).as_str())
                .parse::<u32>()
                .unwrap();
            write(
                format!("{HYBRID_DIR}/{}/pid", container_id),
                pid.to_string(),
            )
            .expect("Unable to write status to file.");
            info!("we are sure the file exists");
            write(format!("{HYBRID_DIR}/{}/status", container_id), "RUNNING")
                .expect("Unable to write status to file.");
            resp.pid = pid;
            info!("pid from main {}", resp.pid);
            info!("create {:?}", resp);
            Ok(resp)
        } else {
            // create hybrid container
            let req = GetImageRequest {
                name: name.to_string(),
            };

            let req: Request<GetImageRequest> = with_namespace!(req, ns);
            let image = ImagesClient::new(channel.clone())
                .get(req)
                .await
                .expect("failed to get image")
                .into_inner()
                .image
                .unwrap();

            let digest = image.target.unwrap().digest;
            let req = ReadContentRequest {
                digest,
                ..Default::default()
            };
            let req: Request<ReadContentRequest> = with_namespace!(req, ns);
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
                    let req = ReadContentRequest {
                        digest: manifest.config().digest().to_string(),
                        ..Default::default()
                    };
                    let req: Request<ReadContentRequest> = with_namespace!(req, ns);
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
                                            container_labels.insert(
                                                "Firmware name".to_string(),
                                                entrypoint[0].clone(),
                                            );
                                        }
                                        _ => panic!("No entrypoint specified"),
                                    };
                                    match config.labels() {
                                        Some(labels) => {
                                            match labels.get("board") {
                                                Some(board) => {
                                                    let f =
                                                        read_to_string(format!("{BOARD}")).unwrap();
                                                    let board_file = f.trim_matches(char::from(0));
                                                    for line in board_file.lines() {
                                                        if line == board {
                                                            container_labels.insert(
                                                                "Board".to_string(),
                                                                board.to_string(),
                                                            );
                                                        } else {
                                                            error!("Image board name label does not match the current board");
                                                            panic!("Image board name label does not match the current board");
                                                            //return Err("Image board name label does not match the current board".to_string());
                                                        }
                                                    }
                                                }
                                                None => {
                                                    error!(
                                                        "board label not set in container image."
                                                    );
                                                    panic!(
                                                        "board label not set in container image."
                                                    );
                                                    // return Err("Board label not set in container image."
                                                    //     .to_string())
                                                }
                                            };
                                            match labels.get("mcu") {
                                                Some(mcu_label) => {
                                                    let mcu_path = match check_mcu_exists(
                                                        REMOTEPROC, mcu_label,
                                                    ) {
                                                        Ok(mcu_path) => mcu_path,
                                                        Err(e) => panic!("{:?}", e), //return Err(e),
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
                                                    //return Err(
                                                    //    "MCU label not set in container image.".to_string()
                                                    //)
                                                    error!("MCU label not set in container image.");
                                                    panic!("MCU label not set in container image.");
                                                }
                                            };
                                        }
                                        _ => {
                                            /*return Err(
                                                "No labels specified. Can't match board name and MCU."
                                                    .to_string(),
                                            )*/
                                            error!("no labels specified. can't match board naem and MCU.");
                                            panic!("no labels specified. can't match board naem and MCU.");
                                        }
                                    };
                                }
                                None => {
                                    error!("couldn't find config.");
                                    panic!("couldn't find config."); //return Err("Couldnt find config".to_string()),
                                }
                            };
                        }
                        Err(_) => {
                            error!("couldn't read blob content.");
                            panic!("couldn't read blob content."); //return Err("couldnt read blob content".to_string()),
                        }
                    }
                    // need to update where remoteproc looks for firmware
                    write(REMOTEPROC_PATH, firmware_path)
                        .expect("Couldn't update firmware default location.");
                    container_labels.insert("Firmware path".to_string(), firmware_path.to_string());
                }
                Err(_) => {
                    error!("failed to read image manifest.");
                    panic!("failed to read image manifest."); //return Err("failed to read image manifest".to_string()),
                }
            }

            // update current container
            let container = Container {
                labels: container_labels,
                ..current_container
            };

            let req = UpdateContainerRequest {
                container: Some(container.clone()),
                update_mask: None,
            };
            let req: Request<UpdateContainerRequest> = with_namespace!(req, ns);
            let _resp = ContainersClient::new(channel.clone())
                .update(req)
                .await
                .expect("failed to update current container")
                .into_inner();

            let ttrpc_address = address + ".ttrpc";
            let publisher = RemotePublisher::new(ttrpc_address).expect("Connect failed");

            let task = TaskCreate {
                container_id: container_id.to_string(),
                bundle: request.bundle,
                rootfs: request.rootfs,
                io: Some(TaskIO {
                    stdin: request.stdin.to_string(),
                    stdout: request.stdout.to_string(),
                    stderr: request.stderr.to_string(),
                    terminal: request.terminal,
                    ..Default::default()
                })
                .into(),
                checkpoint: request.checkpoint.to_string(),
                pid: pid,
                ..Default::default()
            };
            write(format!("{HYBRID_DIR}/{}/status", container_id), "CREATED")
                .expect("Unable to write firmware name to file.");

            write(
                format!("{HYBRID_DIR}/{}/pid", container_id),
                pid.to_string(),
            )
            .expect("Unable to write firmware name to file.");

            publisher
                .publish(
                    Context::default(),
                    "/tasks/create",
                    &ns,
                    Box::new(task.clone()),
                )
                .expect("Can't create task");
            resp.pid = pid;
            info!("create resp: {:?} with pid {:?}", resp, resp.pid);
            Ok(resp)
        }
    }

    #[tokio::main(flavor = "current_thread")]
    async fn state(
        &self,
        _ctx: &TtrpcContext,
        request: StateRequest,
    ) -> TtrpcResult<StateResponse> {
        info!("State request for {:?}", &request);
        let address = self.address.clone();
        let ns = self.namespace.clone();
        let channel = containerd_client::connect(address.clone())
            .await
            .expect("connect failed.");
        let req = GetContainerRequest {
            id: request.id.clone(),
        };

        let req: Request<GetContainerRequest> = with_namespace!(req, ns);
        let current_container = ContainersClient::new(channel.clone())
            .get(req)
            .await
            .expect("failed to get image")
            .into_inner()
            .container
            .unwrap();
        let _name = current_container.clone().image;
        let mut resp = StateResponse::new();
        let pid = read_info(format!("{HYBRID_DIR}/{}/pid", request.id).as_str())
            .parse::<u32>()
            .unwrap();
        let status = read_info(format!("{HYBRID_DIR}/{}/status", request.id).as_str());
        resp.id = request.id;
        resp.pid = pid;
        resp.status = match status.as_str() {
            "UNKNOWN" => Status::UNKNOWN.into(),
            "CREATED" => Status::CREATED.into(),
            "RUNNING" => Status::RUNNING.into(),
            "STOPPED" => Status::STOPPED.into(),
            "PAUSED" => Status::PAUSED.into(),
            "PAUSING" => Status::PAUSING.into(),
            _ => Status::UNKNOWN.into(),
        };
        info!("state {:?}", resp);
        Ok(resp)
    }

    fn wait(&self, _ctx: &TtrpcContext, req: WaitRequest) -> TtrpcResult<WaitResponse> {
        info!("Wait request for {:?}", &req);
        let resp = WaitResponse::new();
        Ok(resp)
    }

    #[tokio::main(flavor = "current_thread")]
    async fn start(
        &self,
        _ctx: &TtrpcContext,
        request: StartRequest,
    ) -> TtrpcResult<StartResponse> {
        info!("Start request for {:?}", &request);
        let ns = self.namespace.clone();
        let address = self.address.clone();
        let channel = containerd_client::connect(address.clone())
            .await
            .expect("connect failed.");

        let req = GetContainerRequest {
            id: request.id.to_string(),
        };

        info!("get container to retreive image");
        let req: Request<GetContainerRequest> = with_namespace!(req, ns);
        let container = ContainersClient::new(channel.clone())
            .get(req)
            .await
            .expect("failed to get image")
            .into_inner()
            .container
            .unwrap();

        let name = container.clone().image;
        let mut resp = StartResponse::new();
        if name.contains("pause") {
            resp.pid = read_info(format!("{HYBRID_DIR}/{}/pause", request.id).as_str())
                .parse::<u32>()
                .unwrap();
            info!("start {:?}", resp);
            Ok(resp)
        } else {
            let labels = container.labels;
            let mcu_path = labels.get("MCU path").unwrap();
            let firmware_name = labels.get("Firmware name").unwrap();
            let _ = File::create(format!("{HYBRID_DIR}/{}", container.id.clone()));

            match check_status(format!("{mcu_path}/{STATE_FILE}").as_str()).unwrap() {
                FirmwareStatus::Offline => {
                    write(format!("{mcu_path}/{FIRMWARE_FILE}"), firmware_name)
                        .expect("Unable to write firmware name to file.");
                    change_status(
                        format!("{mcu_path}/{STATE_FILE}").as_str(),
                        FirmwareStatus::Running,
                    );
                }
                FirmwareStatus::Running => {
                    panic!("Can't start container, a firmware is already running.")
                }
            }
            write(format!("{HYBRID_DIR}/created"), "CREATED")
                .expect("Unable to write firmware name to file.");
            Command::new(format!("{CONSOLE_RPMSG}"))
                .arg(format!("{HYBRID_DIR}/{0}/{0}.log", container.id.clone()))
                .spawn()
                .expect("Failed to run rpmsg console");

            let pid = read_info(format!("{HYBRID_DIR}/{}/pid", request.id).as_str())
                .parse::<u32>()
                .unwrap();
            write(format!("{HYBRID_DIR}/{}/status", request.id), "RUNNING")
                .expect("Unable to write firmware name to file.");
            let task = TaskStart {
                container_id: container.id.to_string(),
                pid: pid,
                ..Default::default()
            };
            let ttrpc_address = address + ".ttrpc";
            let publisher = RemotePublisher::new(ttrpc_address).expect("Connect failed");
            publisher
                .publish(
                    Context::default(),
                    "/tasks/start",
                    &ns,
                    Box::new(task.clone()),
                )
                .expect("Can't create task");

            resp.pid = pid;
            info!("{:?}", resp);
            Ok(resp)
        }
    }

    #[tokio::main(flavor = "current_thread")]
    async fn delete(
        &self,
        _ctx: &TtrpcContext,
        request: DeleteRequest,
    ) -> TtrpcResult<DeleteResponse> {
        info!("Delete request for {:?}", &request);
        let ns = self.namespace.clone();
        let pid = read_info(format!("{HYBRID_DIR}/{}/pid", request.id).as_str())
            .parse::<u32>()
            .unwrap();

        let ts = convert_to_timestamp(Some(OffsetDateTime::now_utc()));
        let task = TaskDelete {
            container_id: request.id.clone(),
            pid: pid,
            exit_status: 0,
            exited_at: Some(ts.clone()).into(),
            ..Default::default()
        };
        let ttrpc_address = self.address.clone() + ".ttrpc";
        let publisher = RemotePublisher::new(ttrpc_address).expect("Connect failed");
        publisher
            .publish(
                Context::default(),
                "/tasks/delete",
                &ns,
                Box::new(task.clone()),
            )
            .expect("Can't delete task");
        let mut resp = DeleteResponse::new();
        resp.pid = pid;
        info!("deleting container");
        std::process::Command::new("ctr")
            .arg("-n")
            .arg(ns)
            .arg("c")
            .arg("rm")
            .arg(request.id.clone())
            .spawn()
            .expect("failed to execute process");
        // if pause container, delete pod
        if Path::new(format!("{HYBRID_DIR}/{}/pause", request.id).as_str()).exists() {
            remove_dir_all(format!(
                "/var/lib/containerd/io.containerd.grpc.v1.cri/sandboxes/{}",
                request.id.clone()
            ))
            .expect("Failed to delete container resources (logs).");

            std::process::Command::new("crictl")
                .arg("rmp")
                .arg(request.id.clone())
                .spawn()
                .expect("failed to execute process");
        }

        remove_dir_all(format!("{HYBRID_DIR}/{}", request.id.clone()))
            .expect("Failed to delete container resources (logs).");
        info!("delete resp {:?}", resp);
        Ok(resp)
    }

    fn stats(&self, _ctx: &TtrpcContext, req: StatsRequest) -> TtrpcResult<StatsResponse> {
        info!("Stats request for {:?}", &req);
        let resp = StatsResponse::new();
        Ok(resp)
    }

    #[tokio::main(flavor = "current_thread")]
    async fn kill(&self, _ctx: &TtrpcContext, request: KillRequest) -> TtrpcResult<Empty> {
        info!("Kill request for {:?}", request);
        let ns = self.namespace.clone();
        let address = self.address.clone();
        let mut pause_kill = false;
        let pause_check = Path::new(format!("{HYBRID_DIR}/{}/pause", request.id).as_str()).exists()
            && Path::new(format!("{HYBRID_DIR}/created").as_str()).exists();
        if pause_check {
            info!("should kill pause container");
            match check_status("/sys/class/remoteproc/remoteproc0/state").unwrap() {
                FirmwareStatus::Offline => {
                    info!("killing pause container");
                    write(format!("{HYBRID_DIR}/{}/status", request.id), "STOPPED")
                        .expect("Unable to write firmware name to file.");

                    pause_kill = true;
                    remove_file(format!("{HYBRID_DIR}/created"))
                        .expect("couldn't remove created hybrid file.");
                    info!("pause after deleted file {:?}", pause_kill);
                }
                FirmwareStatus::Running => (),
            }
        }
        info!("pause kill: {:?}", pause_kill);
        if (request.signal == 15) || (pause_kill) {
            info!("kill:  write stop to status");

            write(format!("{HYBRID_DIR}/{}/status", request.id), "STOPPED")
                .expect("Unable to write firmware name to file.");
            info!("kill:  write done");

            if !pause_kill {
                let channel = containerd_client::connect(address)
                    .await
                    .expect("connect failed.");
                let req = GetContainerRequest {
                    id: request.id.clone(),
                };
                let req: Request<GetContainerRequest> = with_namespace!(req, ns);
                let container = ContainersClient::new(channel.clone())
                    .get(req)
                    .await
                    .expect("failed to get image")
                    .into_inner()
                    .container
                    .unwrap();
                let labels = container.labels;
                let mcu_path = labels.get("MCU path").unwrap();

                match check_status(format!("{mcu_path}/{STATE_FILE}").as_str()).unwrap() {
                    FirmwareStatus::Offline => (),
                    FirmwareStatus::Running => change_status(
                        format!("{mcu_path}/{STATE_FILE}").as_str(),
                        FirmwareStatus::Offline,
                    ),
                }
            } else {
                remove_file(format!("{HYBRID_DIR}/created")).unwrap();
            }
            info!("Kill request for {:?} returns successfully", request);
        }
        Ok(Empty::new())
    }

    fn connect(
        &self,
        _ctx: &TtrpcContext,
        req: api::ConnectRequest,
    ) -> TtrpcResult<api::ConnectResponse> {
        info!("Connect request for {:?}", req);
        Ok(api::ConnectResponse {
            shim_pid: std::process::id(),
            task_pid: std::process::id() + 10,
            ..Default::default()
        })
    }

    fn shutdown(&self, _ctx: &TtrpcContext, _req: api::ShutdownRequest) -> TtrpcResult<api::Empty> {
        info!("Shutdown request");
        self.exit.signal();
        Ok(api::Empty::default())
    }
}

fn main() {
    #[cfg(not(feature = "async"))]
    shim::run::<Service>("io.containerd.hybrid", None)
}
