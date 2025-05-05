use clap::Parser;
use notion_cli_rs::commands::{CliArgs, Commands};
use notion_client::endpoints::{
    Client as NotionClient,
    search::title::{request, response::PageOrDatabase},
};

#[tokio::main]
async fn main() {
    let cli = CliArgs::parse();
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

            println!("the list of databases ({{title}}: {{id}}):");
            for database in response.results {
                if let PageOrDatabase::Database(database) = database {
                    println!(
                        "{}: {}",
                        database.title[0].plain_text().expect("no title is set"),
                        database.id.expect("no id is set")
                    );
                }
            }
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
