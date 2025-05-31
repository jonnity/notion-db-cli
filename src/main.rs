mod commands;
mod operations;

use clap::Parser;
use commands::{CliArgs, Commands};
use operations::{NotionClient, get_example_for_database_property, propery_to_string};
use std::{collections::HashMap, fs::File, process};

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
                    if let Some(file_path) = &args.file {
                        let (property_keys, property_examples): (Vec<String>, Vec<String>) =
                            database
                                .properties
                                .iter()
                                .map(|(name, property)| {
                                    (name.clone(), get_example_for_database_property(property))
                                })
                                .collect();
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
                        database.properties.iter().for_each(|(name, property)| {
                            let property = propery_to_string(property);

                            let name_len = name.chars().count();
                            let type_len = property.chars().count();
                            let max_len = name_len.max(type_len);
                            let pudded_key =
                                format!(" {:<width$} |", name, width = max_len).to_string();
                            let pudded_type =
                                format!(" {:<width$} |", property, width = max_len).to_string();
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
            let file = File::open(&args.file).unwrap();
            let mut reader = csv::ReaderBuilder::new()
                .has_headers(true)
                .trim(csv::Trim::All)
                .from_reader(file);
            let headers = match reader.headers() {
                Ok(headers) => headers.clone(),
                Err(e) => {
                    eprintln!("fail to read headers from csv.");
                    eprintln!("{}", e);
                    process::exit(1);
                }
            };

            for record in reader.records() {
                let record = match record {
                    Ok(record) => record,
                    Err(e) => {
                        eprintln!("fail to read a record in csv.");
                        eprintln!("{}", e);
                        process::exit(1);
                    }
                };
                let mut properties = HashMap::<&str, &str>::new();
                for i in 0..record.len() {
                    let header = headers.get(i).unwrap();
                    let value = record.get(i).unwrap();
                    properties.insert(header, value);
                }
                match client.add_item_to_database(&args.id, properties).await {
                    Ok(()) => (),
                    Err(e) => {
                        eprintln!(
                            "error has occured during creating new database item in a record of csv."
                        );
                        eprintln!("{}", e);
                        process::exit(1);
                    }
                };
            }
        }
    }
}
