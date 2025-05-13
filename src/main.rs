mod commands;
mod operations;

use clap::Parser;
use commands::{CliArgs, Commands};
use operations::{NotionClient, database_to_properties_info};
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
                    let properties = database_to_properties_info(&database); // TODO: improve the text
                    if let Some(file_path) = &args.file {
                        // TODO: construct csv contents and write to the file
                        println!("following contents will be wrote in {}", file_path);
                        let mut property_keys_row = "".to_string();
                        let mut property_example_row = "".to_string();

                        properties.iter().for_each(|property| {
                            property_keys_row += &format!("{}, ", property.name).to_string();
                            property_example_row += &format!("{}, ", property.example).to_string();
                        });
                        println!("{}", property_keys_row);
                        println!("{}", property_example_row);
                    } else {
                        println!("the structure and columns of the database are as follows:");
                        let mut property_keys_row = "|".to_string();
                        let mut property_type_row = "|".to_string();
                        properties.iter().for_each(|property| {
                            let name_len = property.name.chars().count();
                            let type_len = property.r#type.chars().count();
                            let max_len = name_len.max(type_len);
                            let pudded_key =
                                format!(" {:<width$} |", property.name, width = max_len)
                                    .to_string();
                            let pudded_type =
                                format!(" {:<width$} |", property.r#type, width = max_len)
                                    .to_string();
                            property_keys_row += &pudded_key;
                            property_type_row += &pudded_type;
                        });
                        println!("{}", property_keys_row);
                        println!("{}", property_type_row);
                    }
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
            } else if let Some(path) = &args.item.file {
                println!("the contents of {}", path)
            }
        }
    }
}
