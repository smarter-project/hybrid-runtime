use clap::Args;

#[derive(Debug, Args)]
pub struct State {
    /// container ID
    #[clap(required = true)]
    pub container_id: String,
}
