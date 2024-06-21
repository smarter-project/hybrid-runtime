use clap::{Parser, ValueEnum};
use std::path::PathBuf;

#[derive(Debug, ValueEnum, Clone)]
pub enum LogFormat {
    Text,
    Json,
}

/// global options for hybrid-runtime
#[derive(Debug, Parser)]
pub struct GlobalOptions {
    /// enable debug logging
    #[clap(long)]
    pub debug: bool,
    // need a default location to store log
    /// set the log file to write hybrid-runtime logs to.
    #[clap(long)]
    pub log: PathBuf,
    /// set the log format ('text' of 'json') (default: "text")
    #[clap(long, value_enum, default_value_t = LogFormat::Text)]
    pub log_format: LogFormat,
    /// root directory for storage of container state (this should be located in tmpfs)
    //pub root: PathBuf,
    //pub criu: PathBuf,
    //pub systemd_cgroup: String,
    //pub rootless: Rootless,
    /// show help
    #[clap(long, short)]
    pub help: bool,
    /// print the version
    #[clap(long, short)]
    pub version: bool,
}
