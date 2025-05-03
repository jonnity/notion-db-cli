use clap::{Args, Parser, Subcommand};
use reqwest::{
    blocking::{Body, Client},
    header::{HeaderMap, HeaderValue},
};

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

fn main() {
    let cli = Cli::parse();
    match &cli.command {
        Commands::DbList => {
            let mut headers = HeaderMap::new();
            headers.append(
                reqwest::header::CONTENT_TYPE,
                HeaderValue::from_static("application/json"),
            );
            headers.append("Notion-Version", HeaderValue::from_static("2022-06-28"));
            let body = Body::from(
                "{\"filter\": {\"value\": \"database\", \"property\": \"object\"}}".to_string(),
            );
            let response = Client::new()
                .post("https://api.notion.com/v1/search")
                .headers(headers)
                .bearer_auth(&cli.token)
                .body(body)
                .send()
                .unwrap();

            println!("{}", response.text().unwrap());
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
