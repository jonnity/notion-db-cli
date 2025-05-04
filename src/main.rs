use clap::{Args, Parser, Subcommand};
use notion_client::endpoints::{Client as NotionClient, search::title::request};

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
#[command(arg_required_else_help = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Notion integration token
    #[arg(long, env = "NOTION_CLI_RS_TOKEN", hide_env_values = true)]
    token: String,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Show the list of databases
    DbList,
    /// Show the structure of the database matching the id specified by the argument
    DbView(DbViewArgs),
    /// Add the item to the database specified with id
    DbAdd(DbAddArgs),
}

#[derive(Debug, Args)]
struct DbViewArgs {
    /// Target database id
    id: String,
}

#[derive(Debug, Args)]
struct DbAddArgs {
    /// Target database id
    id: String,
    #[clap(flatten)]
    item: DbItemGroup,
}

#[derive(Debug, Args)]
#[group(required = true, multiple = false)]
pub struct DbItemGroup {
    /// specify the item to add with json string
    #[clap(long)]
    json: Option<String>,
    /// specify the item to add with the file contents
    #[clap(long)]
    file_path: Option<String>,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let client = NotionClient::new(cli.token, None).unwrap();
    match &cli.command {
        Commands::DbList => {
            let database_list_request = request::SearchByTitleRequest {
                filter: Some(request::Filter {
                    value: request::FilterValue::Database,
                    property: request::FilterProperty::Object,
                }),
                ..Default::default()
            };

            let response = client
                .search
                .search_by_title(database_list_request)
                .await
                .unwrap();

            println!("{:#?}", response);
        }
        Commands::DbView(args) => {
            println!(
                "the structure of the database whose id is {} will be displayed here",
                args.id
            );
        }
        Commands::DbAdd(args) => {
            println!(
                "the bellow item will be added to the database whose id is {} here",
                args.id
            );
            if let Some(json) = &args.item.json {
                println!("{}", json)
            } else if let Some(path) = &args.item.file_path {
                println!("the contents of {}", path)
            }
        }
    }
}
