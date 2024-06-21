use clap::{Parser, Subcommand};
mod create;
mod delete;
mod features;
mod global;
mod kill;
mod list;
mod logs;
mod run;
mod spec;
mod start;
mod state;
mod utils;

use create::Create;
use delete::Delete;
use features::Features;
use kill::Kill;
use list::List;
use logs::Logs;
use run::Run;
use spec::Spec;
use start::Start;
use state::State;

// since the firmware wont be stored in the default location (/lib/firmware)
// update where remoteproc looks for a firmware
// this is the path to actual file not directory
// Standard: for all paths remove the last /
pub const REMOTEPROC_PATH: &str = "/sys/module/firmware_class/parameters/path";
pub const REMOTEPROC: &str = "/sys/class/remoteproc";
pub const FIRMWARE_LIB: &str = "/lib/firmware";
pub const STATE_FILE: &str = "state";
pub const FIRMWARE_FILE: &str = "firmware";
pub const OVERLAYFS: &str = "/var/lib/containerd/io.containerd.snapshotter.v1.overlayfs";
pub const SNAPSHOT_DB: &str = "metadata.db";
pub const HYBRID_DIR: &str = "/var/lib/hybrid-runtime";
// Need to change the path
pub const CONSOLE_RPMSG: &str = "/home/root/cortexm_console";
pub const BOARD: &str = "/sys/firmware/devicetree/base/model";
pub const MCU: &str = "name";

/*
// container labels

 */

#[derive(Parser, Debug)]
#[clap(
    name = "Hybrid container runtime",
    version,
    about = "Deploy an application across available embedded cores."
)]
#[command(arg_required_else_help = true)]
struct HybridRuntime {
    #[clap(subcommand)]
    basic_cmd: BasicCmd,
    // need to add global options and advance commands  (not now .. baby steps)
}

/// Basic CLI commands defined in OCI spec: create, start, state, kill, delete
#[derive(Debug, Subcommand)]
enum BasicCmd {
    /// Create a container
    Create(Create),
    /// Start a created container
    Start(Start),
    /// Output the state of a container
    State(State),
    /// terminates the container
    Kill(Kill),
    /// Deletes any resources held by the container
    Delete(Delete),
    /// Fetch the logs of a running container
    Logs(Logs),
}
// useful to add logs => hybrid container logs
// separate the rest of the commands from the the oci-cli-spec ones .. implement later
#[derive(Debug, Subcommand)]
enum AdvanceCmd {
    /// Lists containers started by hybrid-runtime
    List(List),
    /// Create a new specification file
    Spec(Spec),
    /// Create and run a container from an image
    Run(Run),
    /// Show the enabled features
    Features(Features),
}

// global options (not implemented .. not needed in our case)

fn main() {
    let namespace = "hybrid";
    let command = HybridRuntime::parse();
    let _command_result = match command.basic_cmd {
        BasicCmd::Create(create_args) => {
            println!("Creating container with ID: {:?}", create_args.container_id);
            match create_args.create(namespace) {
                Ok(_) => println!("container created successfuly"),
                Err(err) => println!("failed to create container. {:?}", err),
            }
        }
        BasicCmd::Start(start_args) => {
            println!("start container {:?}", start_args);
            match start_args.start(namespace) {
                Ok(_) => println!("container started"),
                Err(e) => println!("start err {:?}", e),
            }
        }
        BasicCmd::State(state_args) => println!("state not yet implemented {:?}", state_args),
        BasicCmd::Kill(kill_args) => {
            println!("delete container: {:?}", kill_args);
            let _ = kill_args.delete(namespace);
        }
        BasicCmd::Delete(delete_args) => {
            println!("delete container: {:?}", delete_args);
            let _ = delete_args.delete(namespace);
        }
        BasicCmd::Logs(logs_args) => {
            println!("Container {:?} logs", logs_args.container_id);
            match logs_args.logs(namespace) {
                Ok(_) => println!("Logs finished!"),
                Err(e) => println!("logs err {:?}", e),
            }
        }
    };
}
