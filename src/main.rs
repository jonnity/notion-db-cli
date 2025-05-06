use clap::Parser;
use notion_cli_rs::{
    commands::{CliArgs, Commands},
    operations::NotionClient,
};
use std::process;

#[tokio::main]
async fn main() {
    let cli = CliArgs::parse();
    let client = NotionClient::new(cli.token);
    match &cli.command {
        Commands::DbList => {
            let databases = client.list_database().await;
            match databases {
                Err(e) => {
                    eprintln!("fail to obtain the list of databases.");
                    eprintln!("{}", e);
                    process::exit(1);
                }
                Ok(databases) => {
                    println!("the list of databases ({{title}}: {{id}}):");
                    for database in databases {
                        println!(
                            "{}: {}",
                            database.title[0].plain_text().expect("no title is set"),
                            database.id.expect("no id is set")
                        );
                    }
                }
            }
        }
        Commands::DbView(args) => {
            let database = client.view_database(&args.id).await;
            match database {
                Err(e) => {
                    eprintln!("fail to retrieve the databases information.");
                    eprintln!("{}", e);
                    process::exit(1);
                }
                Ok(database) => {
                    println!("the structure and columns of the database are as follows:");

                    // display the keys of properties
                    print!("|");
                    for (key, _) in &database.properties {
                        print!(" {} |", key);
                    }
                    print!("\n");

                    // display the structure of properties
                    print!("|");
                    for (_, property) in &database.properties {
                        // TODO: properly display information on the property.
                        match property {
                            _ => print!(" String |"),
                            // print!(" {} |",);
                        }
                    }
                    print!("\n")
                }
            }
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
