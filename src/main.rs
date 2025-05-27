mod commands;
mod operations;

use clap::Parser;
use commands::{CliArgs, Commands};
use operations::{NotionClient, database_to_properties_info};
use reqwest::header;
use std::{fs::File, process};

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
                        let property_keys: Vec<String> =
                            properties.iter().map(|p| p.name.clone()).collect();
                        let property_examples: Vec<String> =
                            properties.iter().map(|p| p.example.clone()).collect();
                        let property_keys_csv = property_keys.join(", ");
                        let property_example_csv = property_examples.join(", ");
                        let content = format!("{}\n{}", property_keys_csv, property_example_csv);
                        match std::fs::write(file_path, content) {
                            Ok(_) => println!("Successfully wrote to {}", file_path),
                            Err(e) => {
                                eprintln!("Failed to write to file: {}", e);
                                process::exit(1);
                            }
                        }
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
                let file = File::open(path).unwrap();
                let mut reader = csv::ReaderBuilder::new()
                    .has_headers(true)
                    .trim(csv::Trim::All)
                    .from_reader(file);
                let headers = match reader.headers() {
                    Ok(headers) => headers,
                    Err(e) => {
                        eprintln!("fail to read headers from csv.");
                        eprintln!("{}", e);
                        process::exit(1);
                    }
                };

                let target_db = match client.view_database(&args.id).await {
                    Ok(target_db) => target_db,
                    Err(e) => {
                        eprintln!("fail to retrieve the databases information.");
                        eprintln!("{}", e);
                        process::exit(1);
                    }
                };

                if headers.len().ne(&target_db.properties.len()) {
                    eprintln!("the lengths of keys in Notion DB and in csv header differ.");
                    process::exit(1);
                }
                target_db.properties.iter().for_each(|(key, _property)| {
                    let csv_has_key = headers.iter().any(|header| header.eq(key));
                    if !csv_has_key {
                        eprintln!("the key {} is missing in the csv.", key);
                    }
                });
            }
        }
    }
}
