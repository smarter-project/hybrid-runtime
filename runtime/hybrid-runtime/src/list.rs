use clap::{Args, ValueEnum};

#[derive(ValueEnum, Debug, Clone)]
pub enum ListFormat {
    Table,
    Json,
}

#[derive(Debug, Args)]
pub struct List {
    /// select one of: table or json (default: "table")
    #[arg(long, short, value_enum, default_value_t = ListFormat::Table)]
    pub format: ListFormat,
    /// display only container IDs
    #[arg(long, short)]
    pub quiet: bool,
}
