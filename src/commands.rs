use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
#[command(arg_required_else_help = true)]
pub struct CliArgs {
    #[command(subcommand)]
    pub command: Commands,

    /// Notion integration token
    #[arg(long, env = "NOTION_CLI_RS_TOKEN", hide_env_values = true)]
    pub token: String,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Show the list of databases
    #[clap(name = "db-list")]
    List,
    /// Show the structure of the database matching the id specified by the argument
    #[clap(name = "db-view")]
    View(DbViewArgs),
    /// Add the item to the database specified with id
    #[clap(name = "db-add")]
    Add(DbAddArgs),
    /// Query a database (Filters are not supported yet)
    #[clap(name = "db-query")]
    Query(DbQueryArgs),
}

#[derive(Debug, Args)]
pub struct DbViewArgs {
    /// Target database id
    pub id: String,
    #[clap(long, short)]
    pub file: Option<String>,
}

#[derive(Debug, Args)]
pub struct DbAddArgs {
    /// Target database id
    pub id: String,
    #[clap(long, short)]
    pub file: String,
}

#[derive(Debug, Args)]
pub struct DbQueryArgs {
    /// Target database id
    pub id: String,
    #[clap(long, short)]
    pub query: Option<String>,
}
