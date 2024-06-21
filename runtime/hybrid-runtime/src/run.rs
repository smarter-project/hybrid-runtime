use clap::Args;
use std::path::PathBuf;

#[derive(Debug, Args)]
pub struct Run {
    /// container ID
    #[clap(required = true)]
    pub container_id: String,
    /// path to root of the bundle directory, defaults to the current directory
    #[arg(long, short, default_value = ".")]
    pub bundle: Option<PathBuf>,
    /// path to an AF_UNIX socket which will receive a file descriptor referencing the master end of the console's pseudoterminal
    #[arg(long)]
    pub console_socket: Option<PathBuf>,
    /// detach from the container's process
    #[arg(long, short)]
    pub detach: bool,
    /// do not delete the container after it exits
    #[arg(long)]
    pub keep: bool,
    /// specify the file to write the process id to
    #[arg(long)]
    pub pid_file: Option<PathBuf>,
}
