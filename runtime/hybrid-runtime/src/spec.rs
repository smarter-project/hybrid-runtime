use clap::Args;
use std::path::PathBuf;

#[derive(Debug, Args)]
pub struct Spec {
    /// path to where to store the specification file names "config.json"
    #[arg(long, short)]
    pub path: Option<PathBuf>,
    /// generate a configuration for a rootless container
    #[arg(long)]
    pub rootless: bool,
}
